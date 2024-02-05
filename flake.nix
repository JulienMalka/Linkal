{
  description = "Linkal flake";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:

    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          inherit (pkgs.darwin.apple_sdk.frameworks) Security;
          pkgs = nixpkgs.legacyPackages.${system};
          linkal = with pkgs; rustPlatform.buildRustPackage rec {
            name = "linkal";
            version = "0.1.0";

            src = ./.;

            buildInputs = lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Security ];

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
        in
        {
          defaultPackage = linkal;
          checks = linkal;
          packages = {
            linkal = linkal;
            docker-image = pkgs.dockerTools.buildLayeredImage
              {
                name = "linkal";
                tag = "latest";
                config.Cmd = [ "${linkal}/bin/linkal /data/calendars.json" ];

              };
          };
        }
      ));
}

    
