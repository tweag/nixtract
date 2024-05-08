![Build](https://img.shields.io/github/actions/workflow/status/tweag/nixtract/build_nix.yml
) [![Discord channel](https://img.shields.io/discord/1174731094726295632)](https://discord.gg/53XwX7Ft)

# `nixtract`

A CLI tool to extract the graph of derivations from a Nix flake.

We [invite you](https://discord.gg/53XwX7Ft) to join our [Discord channel](https://discord.com/channels/1174731094726295632/1183682765212897280)! It's a great place to ask questions, share your ideas, and collaborate with other contributors.

## Usage

### Requirements

* Nix >= 2.4
  * experimental feature `nix-command` needs to be enabled: [Nix command - NixOS Wiki](https://wiki.nixos.org/wiki/Nix_command)
  * experimental feature `flakes` needs to be enabled: [Flakes - NixOS Wiki](https://wiki.nixos.org/wiki/Flakes)

### Set up

Get it using Nix

```console
$ nix shell github:tweag/nixtract
```

Or install using `cargo`

```console
cargo install --git https://github.com/tweag/nixtract.git
```

### Usage

```console
$ nixtract --help
A CLI tool and library to extract the graph of derivations from a Nix flake.

Usage: nixtract [OPTIONS] [OUTPUT_PATH]

Arguments:
  [OUTPUT_PATH]
          Write the output to a file instead of stdout or explicitly use `-` for stdout

Options:
  -f, --target-flake-ref <FLAKE_REF>
          The flake URI to extract, e.g. "github:tweag/nixtract"

          [default: nixpkgs]

  -a, --target-attribute-path <ATTRIBUTE_PATH>
          The attribute path to extract, e.g. "haskellPackages.hello", defaults to all derivations in the flake

  -s, --target-system <SYSTEM>
          The system to extract, e.g. "x86_64-linux", defaults to the host system

      --offline
          Run nix evaluation in offline mode

      --n-workers <N_WORKERS>
          Count of workers to spawn to describe derivations

      --pretty
          Pretty print the output

  -v, --verbose...
          Increase logging verbosity

  -q, --quiet...
          Decrease logging verbosity

      --output-schema
          Output the json schema

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Extract nixpkgs graph of derivations

To extract the data from the `nixpkgs` of your flake registry and output to stdout, use:

```console
$ nixtract
```

you can also specify an output file path directly instead

```console
$ nixtract derivations.jsonl
```

in order to extract from a specific flake, use `--target-flake-ref` or `-f`:

```console
$ nixtract --target-flake-ref 'github:nixos/nixpkgs/23.05'
```

in order to extract a specific attribute, use `--target-attribute` or `-a`:

```console
$ nixtract --target-attribute-path 'haskellPackages.hello'
```

in order to extract for a system different from your own, use `--target-system` or `-s`:

```console
$ nixtract --target-system 'x86_64-darwin'
```

in order to only consider runtime dependencies, use `--runtime-only` or `-r`:

```console
$ nixtract --runtime-only
```

### Understanding the output

`nixtract` evaluates Nix code to recursively find all derivations in a flake.
It first finds the top level derivations, basically all derivations you can find with `nix search`.
Then it recurses into the input derivations of any derivation it has found.

Each line of the output is a valid JSON that describes a derivation.
As such, the output is a JSONL file.

The JSON schema of a derivation can be shown like so:

```console
$ nixtract --output-schema
```

## Development

### Set up

#### Using Nix
```console
$ nix develop
```

#### Manually
If Nix is not available, you can install the Rust toolchain manually.

### Under the hood

The overall architecture inside is described in `src/main.rs`:

```
Calling this tool starts a subprocess that list top-level derivations (outputPath + attribute path) to its stderr pipe, see `src/nix/find_attribute_paths.nix`.
This pipe is consumed in a thread that reads each line and populates a vector.
This vector is consumed by rayon threads that will call the `process` function.
This function will call a subprocess that describes the derivation (name, version, license, dependencies, ...), see `src/nix/describe_derivation.nix`.
When describing a derivation, if dependencies are found and have not been already queued for processing, they are added to the current thread's queue, which makes us explore the entire depth of the graph.
Rayon ensures that a thread without work in its queue will steal work from another thread, so we can explore the graph in parallel.

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
