{
  description = "Python library with Rust combinators";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override { extensions = [ "clippy" ]; };

        python = pkgs.python312;

        pythonEnv = python.withPackages (
          ps: with ps; [
            pip
            setuptools
            wheel
            pytest
          ]
        );

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.cargo-watch
            pkgs.rust-analyzer
            pythonEnv
            pkgs.maturin
          ];

          shellHook = ''
            export PYTHONPATH="$(pwd)/python:$PYTHONPATH"
          '';
        };

        packages.default = pkgs.stdenv.mkDerivation {
          pname = "py-combinator";
          version = "0.1.0";
          src = self;

          nativeBuildInputs = [
            rustToolchain
            pythonEnv
            pkgs.maturin
          ];

          buildPhase = ''
            cd $src
            maturin build --release
          '';

          installPhase = ''
            mkdir -p $out/lib/${python.libPrefix}/site-packages
            cp target/wheels/*.whl $out/
            ${python}/bin/pip install --target=$out/lib/${python.libPrefix}/site-packages $out/*.whl
          '';
        };
      }
    );
}
