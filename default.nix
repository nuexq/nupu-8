{ lib, rustPlatform }:
let
  toml = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = "nupu-8";
  inherit (toml) version;

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  cargoBuildOptions = [ "--release" ];

  meta = {
    inherit (toml) homepage description;
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ nuexq ];
    mainProgram = "tracky";
  };
}
