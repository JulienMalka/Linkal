{pkgs ? import <nixpkgs> {} }:
with pkgs;

rustPlatform.buildRustPackage rec {
  pname = "linkal";
  version = "0.1.0";

  src = ./.; 

  buildInputs = [ darwin.apple_sdk.frameworks.Security ];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
