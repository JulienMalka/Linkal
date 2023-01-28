{
  description = "Linkal flake";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:

    rec {

      packages = flake-utils.lib.eachDefaultSystem
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
            linkal = linkal;
          }
        );

      defaultPackage = packages.linkal;
      hydraJobs = {
        linkal = packages.linkal.x86_64-linux;
      };
    };
}
