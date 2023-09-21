# `nixtract`

A CLI tool to extract the graph of derivations from a Nix flake.

## Usage

### Requirements

* Nix >= 2.4
  * experimental feature `nix-command` needs to be enabled: [Nix command - NixOS Wiki](https://nixos.wiki/wiki/Nix_command)
  * experimental feature `flakes` needs to be enabled: [Flakes - NixOS Wiki](https://nixos.wiki/wiki/Flakes)
* Python >= 3.10

### Set up

```console
$ pip install git+https://github.com/tweag/nixtract.git
```

### Usage

```console
$ nixtract --help
Usage: nixtract [OPTIONS] OUTFILE

  Extract the graph of derivations from a flake as JSONL.

  OUTFILE is the path to the output file to write to, use "-" to write to
  stdout.

Options:
  --target-flake-ref TEXT  The reference of the target Nix Flake
  --target-system TEXT     The system in which to evaluate the derivations
  --n-workers INTEGER      Count of workers to spawn to describe the stream of
                           found derivations
  --offline                Pass --offline to Nix commands
  -v, --verbose            Increase verbosity
  --help                   Show this message and exit.
```

### Extract nixpkgs graph of derivations

To extract the data from the `nixpkgs` of your flake registry and output to stdout, use:

```console
$ nixtract -
```

you can specify a file path directly instead

```console
$ nixtract derivations.jsonl
```

### Understanding the output

`nixtract` evaluates Nix code to recursively find all derivations in a flake.
It first finds the top level derivations, basically all derivations you can find with `nix search`.
Then it recurses into the input derivations of any derivation it has found.

Each line of the output is a valid JSON that describes a derivation.
As such, the output is a JSONL file.

The JSON schema of a derivation can be shown like so:

```console
$ python -c 'import nixtract.model; print(nixtract.model.Derivation.schema_json(indent=2))'
```

## Development

### Set up

```console
$ poetry install
```
