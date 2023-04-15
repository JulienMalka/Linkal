let
  pkgs = import <nixpkgs> { };

in
pkgs.mkShell {
  buildInputs = [ pkgs.cargo pkgs.rustc ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
