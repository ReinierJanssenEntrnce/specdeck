# NixOS module for SpecDeck's server side on one machine with a public IP: the signalling service
# behind Caddy (TLS from Let's Encrypt), and coturn as the TURN relay. Mirrors deploy/k8s/base;
# see deploy/hetzner/README.md.
#
#   services.specdeck-signalling = { enable = true; domain = "signal.example.com"; };
#
# Apps then connect to wss://<domain>/signal. The TURN secret is generated on the machine on first
# start and never leaves it.
self:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.specdeck-signalling;
  secretFile = "/var/lib/specdeck-turn/secret";
in
{
  options.services.specdeck-signalling = {
    enable = lib.mkEnableOption "the SpecDeck signalling service and TURN relay";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.signalling;
      defaultText = lib.literalExpression "specdeck.packages.\${system}.signalling";
      description = "The signalling service package.";
    };

    domain = lib.mkOption {
      type = lib.types.str;
      example = "signal.example.com";
      description = ''
        Public name of the service. Caddy gets a certificate for it, apps connect to
        wss://<domain>/signal, and TURN is reached at <domain>:3478, so its DNS record must point
        at this machine.
      '';
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Local port of the signalling service, behind Caddy.";
    };

    turn = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = ''
          Whether to run coturn. Without it, viewers that cannot reach the host directly (strict
          NATs, firewalls) cannot connect.
        '';
      };
      minPort = lib.mkOption {
        type = lib.types.port;
        default = 49160;
        description = "First relay port.";
      };
      maxPort = lib.mkOption {
        type = lib.types.port;
        default = 49179;
        description = ''
          Last relay port. Each relayed viewer uses about two, so the default covers about ten at
          once.
        '';
      };
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Whether to open HTTP(S), TURN and the relay ports in the firewall.";
    };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        systemd.services.specdeck-signalling = {
          description = "SpecDeck signalling service";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ] ++ lib.optional cfg.turn.enable "specdeck-turn-secret.service";
          requires = lib.optional cfg.turn.enable "specdeck-turn-secret.service";
          environment = {
            SPECDECK_LISTEN = "127.0.0.1:${toString cfg.port}";
            # Caddy sits in front and adds the client's address to X-Forwarded-For, which the rate
            # limits use.
            SPECDECK_TRUSTED_PROXIES = "1";
            RUST_LOG = "info";
          }
          // lib.optionalAttrs cfg.turn.enable {
            SPECDECK_TURN_URLS = "turn:${cfg.domain}:3478?transport=udp";
            # A relayed session refreshes its allocation with these credentials: cover a long day.
            SPECDECK_TURN_TTL_SECONDS = "43200";
          };
          script =
            lib.optionalString cfg.turn.enable ''
              SPECDECK_TURN_SECRET=$(< "$CREDENTIALS_DIRECTORY/turn-secret")
              export SPECDECK_TURN_SECRET
            ''
            + ''
              exec ${lib.getExe cfg.package}
            '';
          serviceConfig = {
            DynamicUser = true;
            LoadCredential = lib.optional cfg.turn.enable "turn-secret:${secretFile}";
            Restart = "always";
            CapabilityBoundingSet = "";
            NoNewPrivileges = true;
            PrivateDevices = true;
            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;
            ProtectKernelTunables = true;
            ProtectKernelModules = true;
            ProtectControlGroups = true;
            ProtectClock = true;
            ProtectHostname = true;
            ProtectProc = "invisible";
            RestrictAddressFamilies = [
              "AF_INET"
              "AF_INET6"
            ];
            RestrictNamespaces = true;
            RestrictRealtime = true;
            LockPersonality = true;
            MemoryDenyWriteExecute = true;
            SystemCallArchitectures = "native";
            SystemCallFilter = [ "@system-service" ];
            MemoryMax = "128M";
          };
        };

        services.caddy = {
          enable = true;
          virtualHosts.${cfg.domain}.extraConfig = ''
            @signalling path /signal /health
            handle @signalling {
              reverse_proxy 127.0.0.1:${toString cfg.port}
            }
            handle {
              respond 404
            }
          '';
        };

        networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [
          80
          443
        ];
      }

      (lib.mkIf cfg.turn.enable {
        # The secret coturn checks TURN credentials with and the signalling service signs them with
        # (TURN REST API, use-auth-secret).
        systemd.services.specdeck-turn-secret = {
          description = "Generate SpecDeck's TURN secret";
          before = [ "coturn.service" ];
          requiredBy = [ "coturn.service" ];
          serviceConfig = {
            Type = "oneshot";
            RemainAfterExit = true;
            StateDirectory = "specdeck-turn";
            StateDirectoryMode = "0750";
            Group = "turnserver";
            UMask = "0027";
          };
          script = ''
            if [ ! -s ${secretFile} ]; then
              ${lib.getExe pkgs.openssl} rand -hex 32 > ${secretFile}
            fi
          '';
        };

        services.coturn = {
          enable = true;
          listening-port = 3478;
          min-port = cfg.turn.minPort;
          max-port = cfg.turn.maxPort;
          realm = "specdeck";
          use-auth-secret = true;
          static-auth-secret-file = secretFile;
          no-cli = true;
          # The app reaches TURN over UDP only (libjuice has no TURN over TCP or TLS).
          no-tcp = true;
          no-tls = true;
          no-dtls = true;
          no-tcp-relay = true;
          extraConfig = ''
            fingerprint
            no-multicast-peers
            # Never relay into private networks: TURN credentials must not give access to anything
            # on or next to this machine. The machine's own public address stays allowed, so two
            # relayed peers can meet on it.
            denied-peer-ip=0.0.0.0-0.255.255.255
            denied-peer-ip=127.0.0.0-127.255.255.255
            denied-peer-ip=10.0.0.0-10.255.255.255
            denied-peer-ip=172.16.0.0-172.31.255.255
            denied-peer-ip=192.168.0.0-192.168.255.255
            denied-peer-ip=169.254.0.0-169.254.255.255
            denied-peer-ip=100.64.0.0-100.127.255.255
            denied-peer-ip=::1
            denied-peer-ip=fc00::-fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff
            denied-peer-ip=fe80::-febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff
          '';
        };

        networking.firewall = lib.mkIf cfg.openFirewall {
          allowedUDPPorts = [ 3478 ];
          allowedUDPPortRanges = [
            {
              from = cfg.turn.minPort;
              to = cfg.turn.maxPort;
            }
          ];
        };
      })
    ]
  );
}
