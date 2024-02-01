{
  outputs = { flake-utils, poetry2nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = poetry2nix.inputs.nixpkgs.outputs.legacyPackages.${system};
        inherit (poetry2nix.lib.mkPoetry2Nix { inherit pkgs; }) mkPoetryApplication;
      in
      {
        packages.default = mkPoetryApplication {
          projectDir = ./.;
          meta.mainProgram = "nixtract";
        };
      });
}
