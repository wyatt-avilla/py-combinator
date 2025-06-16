# Py Combinator

![Rust Edition](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fwyatt-avilla%2Fpy-combinator%2Frefs%2Fheads%2Fmain%2Fpy-combinator%2FCargo.toml&query=%24.package.edition&label=Rust%20Edition&color=%23F74C00)
![Python Version from PEP 621 TOML](https://img.shields.io/python/required-version-toml?tomlFilePath=https%3A%2F%2Fraw.githubusercontent.com%2Fwyatt-avilla%2Fpy-combinator%2Frefs%2Fheads%2Fmain%2Fpy-combinator%2Fpyproject.toml&label=Python)
![PyPI - Version](https://img.shields.io/pypi/v/py-combinator)
![PyPI - Wheel](https://img.shields.io/pypi/wheel/py-combinator)
![MyPy](https://img.shields.io/badge/Mypy-Check-blue?logo=python)
![Clippy](https://img.shields.io/badge/Clippy-Check-green?logo=rust)
![Nix Flake Check](https://img.shields.io/static/v1?label=Nix%20Flake&message=Check&style=flat&logo=nixos&colorB=9173ff&logoColor=CAD3F5)

**py-combinator** is a high-performance Python library implemented in Rust that
provides statically typed
[iterator combinators](https://learning-rust.github.io/docs/combinators/) for
chainable functional operations on iterables.

## Installation

### Via Python Package Vendors

#### uv (and pip)

```sh
uv pip install py-combinator
```

#### Poetry

```sh
poetry add py-combinator
```

### Via Nix Flakes

```nix
{
  description = "Minimal example using py-combinator";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.05";
    py-combinator.url = "github:wyatt-avilla/py-combinator";
  };
  outputs = { nixpkgs, py-combinator, ... }:
    let
      system = "x86_64-linux";  # or your target system
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.${system}.default = pkgs.python312.withPackages (ps: [
        py-combinator.packages.${system}.default
      ]);
    };
}
```

## Similar Libraries

- [Chemical](https://github.com/Pebaz/Chemical)
- [f_it](https://github.com/clbarnes/f_it)
- [qlist](https://github.com/WitoldFracek/qlist)
