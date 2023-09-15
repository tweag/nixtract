import logging
from typing import IO

import click

import nixtract
import nixtract.subprocesses.extract


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
    "--verbose",
    "-v",
    is_flag=True,
    help="Increase verbosity",
)
def cli(
    outfile: IO[str],
    target_flake_ref: str,
    target_attribute_path: str,
    target_system: str,
    n_workers: int,
    offline: bool,
    verbose: bool,
):
    """
    Extract the graph of derivations from a flake as JSONL.

    OUTFILE is the path to the output file to write to, use "-" to write to stdout.
    """
    logger = logging.getLogger(__name__)
    logger.addHandler(logging.StreamHandler())
    if verbose:
        logger.setLevel(logging.INFO)

    nixtract.subprocesses.extract.extract(
        outfile=outfile,
        target_flake_ref=target_flake_ref,
        target_system=target_system,
        target_attribute_path=target_attribute_path,
        n_workers=n_workers,
        offline=offline,
    )
