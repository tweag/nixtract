{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          pname = "nixtract";
          src = ./.;

          # nixtract uses the reqwest crate to query for narinfo on the substituters.
          # reqwest depends on openssl.
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; ([
            openssl
          ] ++ lib.optionals stdenv.isDarwin (with darwin; [
            apple_sdk.frameworks.SystemConfiguration
            libiconv
          ]));
        };
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
            cargo-flamegraph
            cargo-dist

            pkg-config
            openssl
          ] ++ lib.optionals stdenv.isDarwin (with darwin; [
            darwin.apple_sdk.frameworks.SystemConfiguration
            libiconv
          ]);

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
