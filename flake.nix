{
  inputs.nix.url = "github:tweag/nix/nix-c-bindings";

  outputs = { flake-utils, nixpkgs, nix, ... }:
    let
      system = "x86_64-linux";
      # packages from nixpkgs
      pkgs = nixpkgs.legacyPackages.${system};
      # packages from the `nix` input flake (not nixpkgs)
      nixPkgs = nix.packages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          # fork of Nix with C bindings
          nixPkgs.nix
          # rust
          pkgs.clang
          pkgs.llvmPackages.bintools
          pkgs.rustup
        ];
        RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
        # https://github.com/rust-lang/rust-bindgen#environment-variables
        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
        shellHook = ''
          export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
          export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
        '';
        # Add precompiled library to rustc search path
        RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
          # add libraries here (e.g. pkgs.libvmi)
        ]);
        # Add glibc, clang, glib and other headers to bindgen search path
        BINDGEN_EXTRA_CLANG_ARGS =
          # Includes with normal include path
          (builtins.map (a: ''-I"${a}/include"'') [
            # add dev libraries here (e.g. pkgs.libvmi.dev)
            pkgs.glibc.dev
          ])
          # Includes with special directory paths
          ++ [
            ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
            ''-I"${pkgs.glib.dev}/include/glib-2.0"''
            ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
          ];
      };
    };
}
