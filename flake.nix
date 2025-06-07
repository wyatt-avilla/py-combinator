{
  description = "Python library with Rust combinators";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
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
    let
      projectName = "py-combinator";
      projectVersion = "0.1.0";
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override { extensions = [ "clippy" ]; };

        python = pkgs.python312;

        pythonPackage = pkgs.python312Packages.buildPythonPackage {
          name = projectName;
          version = projectVersion;
          pyproject = true;
          src = self;

          cargoDeps = pkgs.rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };
          nativeBuildInputs = [
            pkgs.rustPlatform.cargoSetupHook
            pkgs.rustPlatform.maturinBuildHook
            rustToolchain
            pkgs.maturin
          ];
          buildInputs = [ python ];
          buildAndTestSubdir = "py-combinator";

          pythonImportsCheck = [ "py_combinator" ];
        };

        pythonEnv = python.withPackages (ps: [ pythonPackage ] ++ (with ps; [ ipython ]));
      in
      {
        packages = {
          default = pythonPackage;
          inherit pythonEnv;
        };

        apps = {
          default = {
            type = "app";
            program = "${pythonEnv}/bin/ipython";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            (python.withPackages (
              ps: with ps; [
                pip
                setuptools
                wheel
                pytest
              ]
            ))
            rustToolchain
            pkgs.cargo-watch
            pkgs.cargo-expand
            pkgs.rust-analyzer
            pkgs.maturin
            pkgs.ruff
            pkgs.mypy
          ];

          shellHook = ''
            export PYTHONPATH="$(pwd)/python:$PYTHONPATH"
          '';
        };
      }
    );
}
