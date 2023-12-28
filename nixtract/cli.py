import logging
from typing import IO

import click

import nixtract


@click.command()
@click.argument(
    "outfile",
    type=click.File("wt"),
)
@click.option(
    "--target-flake-ref",
    default="github:nixos/nixpkgs/master",
    help="The reference of the target Nix Flake",
)
@click.option(
    "--target-attribute-path",
    default="",
    help="The attribute path of the provided attribute set to evaluate",
)
@click.option(
    "--target-system",
    default="x86_64-linux",
    help="The system in which to evaluate the derivations",
)
@click.option(
    "--n-workers",
    default=1,
    type=int,
    help="Count of workers to spawn to describe the stream of found derivations",
)
@click.option(
    "--offline",
    is_flag=True,
    help="Pass --offline to Nix commands",
)
@click.option(
    "--log-level",
    type=click.Choice(["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]),
    help="Set the log level",
    default="INFO",
)
@click.option(
    "--fast",
    is_flag=True,
    help="[EXPERIMENTAL] _Blazingly_ fast using C FFI bindings. See README to install.",
)
def cli(
    outfile: IO[str],
    target_flake_ref: str,
    target_attribute_path: str,
    target_system: str,
    n_workers: int,
    offline: bool,
    log_level: str,
    fast: bool,
):
    """
    Extract the graph of derivations from a flake as JSONL.

    OUTFILE is the path to the output file to write to, use "-" to write to stdout.
    """
    logger = logging.getLogger()
    logger.addHandler(logging.StreamHandler())
    logger.setLevel(logging._nameToLevel[log_level])

    if fast:
        import nixtract.c_bindings.extract

        if offline is True:
            logger.warning("--offline is not used in fast mode")

        nixtract.c_bindings.extract.extract(
            outfile=outfile,
            target_flake_ref=target_flake_ref,
            target_system=target_system,
            target_attribute_path=target_attribute_path,
        )
        return
    else:
        import nixtract.subprocesses.extract

        nixtract.subprocesses.extract.extract(
            outfile=outfile,
            target_flake_ref=target_flake_ref,
            target_attribute_path=target_attribute_path,
            target_system=target_system,
            n_workers=n_workers,
            offline=offline,
        )
