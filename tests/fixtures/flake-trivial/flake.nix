{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { flake-utils, nixpkgs, ... }:
  flake-utils.lib.eachDefaultSystem (system: 
  let pkgs = nixpkgs.legacyPackages.${system};
  in {
      packages.default = builtins.derivation {
        name = "trivial-1.0";
        system = system;
        outputs = ["out"];
        builder = "/bin/sh";
        args = ["-c" "echo trivial > $out"];
      };
    });
}
