![Build](https://img.shields.io/github/actions/workflow/status/tweag/nixtract/ci.yml
) [![Discord channel](https://img.shields.io/discord/1174731094726295632)](https://discord.gg/53XwX7Ft)

# `nixtract`

A CLI tool to extract the graph of derivations from a Nix flake.

We [invite you](https://discord.gg/53XwX7Ft) to join our [Discord channel](https://discord.com/channels/1174731094726295632/1183682765212897280)! It's a great place to ask questions, share your ideas, and collaborate with other contributors.

## Usage

### Requirements

* Nix >= 2.4
  * experimental feature `nix-command` needs to be enabled: [Nix command - NixOS Wiki](https://nixos.wiki/wiki/Nix_command)
  * experimental feature `flakes` needs to be enabled: [Flakes - NixOS Wiki](https://nixos.wiki/wiki/Flakes)

### Set up

Get it using Nix

```console
$ nix shell github:tweag/nixtract
```

or install in your Python environment:

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
  --target-flake-ref TEXT       The reference of the target Nix Flake
  --target-attribute-path TEXT  The attribute path of the provided
                                attribute set to evaluate. If empty, the
                                entire attribute set is evaluated
  --target-system TEXT          The system in which to evaluate the
                                derivations
  --n-workers INTEGER           Count of workers to spawn to describe the
                                stream of found derivations
  --offline                     Pass --offline to Nix commands
  -v, --verbose                 Increase verbosity
  --help                        Show this message and exit.
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

in order to extract from a specific flake, use `--target-flake-ref`:

```console
$ nixtract --target-flake-ref 'github:nixos/nixpkgs/23.05' -
```

in order to extract from a specific attribute, use `--target-attribute`:

```console
$ nixtract --target-attribute-path 'haskellPackages.hello' -
```

in order to extract for a specific system, use `--target-system`:

```console
$ nixtract --target-system 'x86_64-darwin' -
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

Requires a Python environment (^3.10) and `poetry`.
```console
$ poetry install
```

### Under the hood

The overall architecture inside is described in `nixtract/cli.py`:

```
Calling this tool starts a subprocess that list top-level derivations (outputPath + attribute path) to its stderr pipe, see `./find-attribute-paths.nix`.
This pipe is consumed in a thread (`finder_output_reader`) that reads each line and feeds found attribute paths to a queue.
This queue is consumed by another thread (`queue_processor`) that will call a subprocess that describes the derivation (name, version, license, dependencies, ...), see `./describe-derivation.nix`.
When describing a derivation, if dependencies are found and have not been already queued for processing, they are added to the queue as well, which makes us explore the entire depth of the graph.

The whole system stops once
- all top-level attribute paths have been found
- all derivations from that search have been processed
- all dependencies have been processed

Glossary:
- output path: full path of the realization of the derivation in the Nix store.
               e.g. /nix/store/py9jjqsgsya5b9cpps64gchaj8lq2h5i-python3.10-versioneer-0.28
- attribute path: path from the root attribute set to get the desired value.
                  e.g. python3Derivations.versioneer
```
