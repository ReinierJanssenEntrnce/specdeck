{
  description = "SpecDeck: run a coding agent with your team watching along";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Only for installing the sharing service's server (deploy/hetzner).
    disko = {
      url = "github:nix-community/disko";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Linux only: macOS installs through Homebrew (design decision 13).
  outputs =
    {
      self,
      nixpkgs,
      disko,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forEachSystem (pkgs: rec {
        specdeck = pkgs.callPackage ./nix/package.nix { src = self; };
        signalling = pkgs.callPackage ./nix/signalling.nix { src = self; };
        default = specdeck;
      });

      apps = forEachSystem (pkgs: {
        default = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${pkgs.stdenv.hostPlatform.system}.specdeck;
        };
      });

      overlays.default = final: _prev: {
        specdeck = final.callPackage ./nix/package.nix { src = self; };
      };

      # The signalling service and TURN relay; see nix/signalling-module.nix.
      nixosModules.signalling = import ./nix/signalling-module.nix self;

      # The sharing service at specdeck.hartj.es; see deploy/hetzner/README.md.
      nixosConfigurations.specdeck-signal = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          disko.nixosModules.disko
          self.nixosModules.signalling
          ./deploy/hetzner/configuration.nix
        ];
      };

      checks = forEachSystem (pkgs: {
        specdeck = self.packages.${pkgs.stdenv.hostPlatform.system}.specdeck;
        signalling = self.packages.${pkgs.stdenv.hostPlatform.system}.signalling;
      });
    };
}
