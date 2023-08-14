# `nixtract`

A CLI tool to extract the graph of derivations from a Nix flake.

## Usage

### Set up

```console
pip install git+https://github.com/tweag/nixtract.git
```

### Extract nixpkgs graph of derivations

To extract the data from a specific nixpkgs, use:

```console
nixtract ./derivations.jsonl
```

To write to stdout, use `-` instead of a file path

```console
nixtract -
```

To learn more about the available options:

```console
nixtract --help
```

## Development

### Set up

Use the Nix development shell provided in this repository, see `../README.md#use-nix`

```console
poetry install
```
