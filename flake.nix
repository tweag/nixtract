{
  outputs = { flake-utils, nixpkgs, nix, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      packages.${system}.default = pkgs.poetry2nix.mkPoetryApplication {
        projectDir = ./.;
      };
    };
}
