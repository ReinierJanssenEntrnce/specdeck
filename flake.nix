{
  description = "SpecDeck: run a coding agent with your team watching along";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  # Linux only: macOS installs through Homebrew (design decision 13).
  outputs =
    { self, nixpkgs }:
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

      checks = forEachSystem (pkgs: {
        specdeck = self.packages.${pkgs.stdenv.hostPlatform.system}.specdeck;
      });
    };
}
