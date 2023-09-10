{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { flake-utils, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        pkg1 = pkgs.stdenv.mkDerivation {
          name = "pkg1";
          src = ./pkg1;
          buildInputs = [ pkgs.bash ];
          installPhase = ''
            mkdir -p $out/bin
            cp $src/* $out/bin
          '';
        };

        pkg2 = pkgs.stdenv.mkDerivation {
          name = "pkg2";
          src = ./pkg2;
          buildInputs = [ pkgs.bash pkg1 ];
          buildPhase = ''
            pkg1
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp $src/* $out/bin
          '';
        };

      in
      {
        packages.default = pkg2;
      }
    );
}
