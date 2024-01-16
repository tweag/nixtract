{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.hello = {
    url = "https://git.savannah.gnu.org/git/hello.git";
    flake = false;
  };

  outputs = { flake-utils, nixpkgs, hello, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "hello";
          version = "0.1.0";

          src = pkgs.fetchFromGitHub {
            owner = "hello-lang";
            repo = "Rust";
            rev = "8e8bd39a444f6d6c7b01046a6b0600273911ac58";
            hash = "sha256-w3IRfqPsLFAY7OpLRaJpnIUIMhtJ71xi1PJGNttb9EQ=";
          };

          cargoHash = "sha256-eouoalg2VO6SCG9oaCPqmlfWnKI+uy6ggPX7IKG1hz4=";

          meta = with pkgs.lib; {
            description = "Hello World! in Rust";
            homepage = "https://github.com/hello-lang/Rust";
            license = licenses.unlicense;
            maintainers = [ ];
          };

        };
      }
    );
}
