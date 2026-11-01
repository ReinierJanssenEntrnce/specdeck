# SpecDeck's sharing service on a Hetzner Cloud server: signalling behind Caddy, and coturn.
# Kept as small as NixOS gets: no documentation, no desktop bits, SSH with keys only.
# Installing and updating: see README.md.
{ modulesPath, ... }:

{
  imports = [
    (modulesPath + "/profiles/minimal.nix")
    (modulesPath + "/profiles/qemu-guest.nix")
    ./disk.nix
  ];

  services.specdeck-signalling = {
    enable = true;
    domain = "specdeck.hartj.es";
  };

  networking.hostName = "specdeck-signal";
  # Hetzner Cloud hands out the IPv4 address over DHCP. IPv6 is left out: it would need the
  # server's /64 configured statically, and TURN works without it.
  networking.useNetworkd = true;
  networking.useDHCP = true;

  boot.loader.grub = {
    efiSupport = true;
    efiInstallAsRemovable = true;
  };

  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      KbdInteractiveAuthentication = false;
      PermitRootLogin = "prohibit-password";
    };
  };
  # Your public SSH key(s), one per line; the file has to be committed for the flake to see it.
  users.users.root.openssh.authorizedKeys.keyFiles = [ ./authorized_keys ];

  nix.settings.experimental-features = [
    "nix-command"
    "flakes"
  ];
  nix.gc = {
    automatic = true;
    dates = "weekly";
    options = "--delete-older-than 14d";
  };
  nix.optimise.automatic = true;
  services.journald.extraConfig = "SystemMaxUse=200M";

  time.timeZone = "UTC";
  system.stateVersion = "26.05";
}
