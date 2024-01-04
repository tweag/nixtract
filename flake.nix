{
  outputs = { flake-utils, nixpkgs, nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.poetry2nix.mkPoetryApplication {
          projectDir = ./.;
          meta.mainProgram = "nixtract";
        };
      });
}
