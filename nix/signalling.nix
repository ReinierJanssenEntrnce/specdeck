# The signalling service (matchmaking and TURN credentials), without the app. Deployed with the
# NixOS module in signalling-module.nix.
{
  lib,
  src,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "specdeck-signalling";
  version = (lib.importTOML (src + "/Cargo.toml")).workspace.package.version;
  inherit src;

  cargoLock.lockFile = src + "/Cargo.lock";

  cargoBuildFlags = [
    "-p"
    "specdeck-signalling"
  ];
  cargoTestFlags = [
    "-p"
    "specdeck-signalling"
  ];

  meta = {
    description = "SpecDeck matchmaking service and TURN credentials";
    homepage = "https://github.com/nickhartjes/specdeck";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "specdeck-signalling";
  };
}
