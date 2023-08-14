"""
Extraction of a graph of derivations in a flake.

Implements a CLI using `click` to be used in the terminal.

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
"""  # noqa
import copy
import json
import logging
import multiprocessing.pool
import os
import pathlib
import subprocess
import sys
from queue import Empty
from threading import Lock
from typing import IO

import click

from nixtract import model
from nixtract.threading import thread


@thread
def finder_output_reader(
    pipe_in: IO[bytes],
    queue: multiprocessing.Queue,
    queued_output_paths: set[str],
    logger: logging.Logger,
):
    """Read lines from finder standard output to push found attribute paths to queue"""
    with pipe_in:
        line_bytes: bytes
        # read incoming lines
        for line_bytes in iter(pipe_in.readline, b""):
            line: str = line_bytes.decode()

            if not line.startswith("trace: "):
                sys.stderr.write(line)
                continue

            # remove beginning "trace:"
            trace = line[6:]

            try:
                # FIXME use pydantic
                parsed_found_derivations: list[dict[str, str]] = json.loads(trace)[
                    "foundDrvs"
                ]
            except Exception:
                # fail silently, most likely some other trace from nixpkgs
                sys.stderr.write(line)
                continue

            # push found derivations to queue if they haven't been queued already
            for found_derivation in parsed_found_derivations:
                attribute_path = found_derivation.get("attributePath")
                output_path = found_derivation.get("outputPath")
                if attribute_path is None or output_path is None:
                    logger.warning("Wrong derivation: %s", json.dumps(found_derivation))
                    continue
                if output_path not in queued_output_paths:
                    queue.put(attribute_path)
                    queued_output_paths.add(output_path)


def process_attribute_path(
    pipe_out: IO[str],
    pipe_out_lock: Lock,
    queue: multiprocessing.Queue,
    queued_output_paths: set[str],
    visited_output_paths: set[str],
    finder_env: dict,
    offline: bool,
    attribute_path: str,
    logger: logging.Logger,
):
    """
    Describe a derivation, write results to output pipe and add potential derivations
    to process to queue
    """

    # evaluate value at the attribute path using Nix
    env = copy.deepcopy(finder_env)
    env["TARGET_ATTRIBUTE_PATH"] = attribute_path
    description_process = subprocess.run(
        args=[
            arg
            for arg in [
                "nix",
                "eval",
                "--offline" if offline else None,
                "--extra-experimental-features",
                "nix-command flakes",
                "--json",
                "--file",
                str(pathlib.Path(__file__).parent.joinpath("describe-derivation.nix")),
            ]
            if arg is not None
        ],
        capture_output=True,
        # write to file passed as argument (or stdout)
        env=env,
    )

    description_str = description_process.stdout.decode()
    if len(description_str) == 0:
        logger.warning("Empty evaluation: %s", attribute_path)
        logger.error(description_process.stdout)
        return True

    # add its inputs to the queue if they have not been processed
    try:
        description: model.Derivation = model.Derivation.parse_raw(description_str)
    except Exception as e:
        logger.warning(
            "Failed to parse to model: attribute_path=%s, str=",
            attribute_path,
            description_str,
        )
        logger.error(description_process.stdout)
        raise e from e

    if description.output_path is None:
        logger.warning("Ignore empty outputPath: %s", attribute_path)
        return True

    # write to output pipe
    with pipe_out_lock:
        pipe_out.write(description.json(by_alias=False))
        pipe_out.write("\n")

    visited_output_paths.add(description.output_path)
    for build_input in description.build_inputs:
        output_path = build_input.output_path
        attribute_path = build_input.attribute_path
        if output_path is not None and output_path not in queued_output_paths:
            queue.put(attribute_path)
            queued_output_paths.add(output_path)

    # return success
    return True


@thread
def queue_processor(
    outfile: IO[str],
    outfile_lock: Lock,
    finder_process: subprocess.Popen,
    pool: multiprocessing.pool.ThreadPool,
    queue: multiprocessing.Queue,
    queued_output_paths: set[str],
    visited_output_paths: set[str],
    finder_env: dict,
    offline: bool,
    logger: logging.Logger,
):
    """Continuously process attribute paths in the queue to the pool"""

    # list of ongoing evaluations in the pool
    pool_promises: list[multiprocessing.pool.AsyncResult] = []

    def try_queue_get():
        try:
            return queue.get(block=True, timeout=1)
        except Empty:
            return None

    # main loop of task
    while (
        # there is an attribute path to process
        # FIXME this is active polling, a notification system would save resources
        (attribute_path := try_queue_get()) is not None
        # there are attribute paths to process
        or not queue.empty()
        # the finder process is still running
        or finder_process.poll() is None
        # there are promises which have yet to resolve
        or len(pool_promises) > 0
    ):
        # filter promises to keep unresolved ones
        for promise in pool_promises:
            if promise.ready():
                # evaluation is done, remove it
                pool_promises.remove(promise)
                # check success
                try:
                    promise.get()
                except Exception as e:
                    logger.exception(e)

        logger.info(
            "visited=%s,queue=%s,promises=%s",
            len(visited_output_paths),
            queue.qsize(),
            len(pool_promises),
        )
        # if no attribute path in the queue, look for the next one
        if attribute_path is None:
            continue

        # process attribute path
        promise = pool.apply_async(
            func=process_attribute_path,
            args=[
                outfile,
                outfile_lock,
                queue,
                queued_output_paths,
                visited_output_paths,
                finder_env,
                offline,
                attribute_path,
                logger,
            ],
        )
        pool_promises.append(promise)

    logger.info("QUEUE PROCESSOR CLOSED")


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

    # overwrite some environment variables
    extra_env = {
        # required to evaluate some Nixpkgs expressions
        "NIXPKGS_ALLOW_BROKEN": "1",
        "NIXPKGS_ALLOW_INSECURE": "1",
        # arguments to the Nix expression can't be passed with `nix eval`
        # we use environment variables instead
        "TARGET_FLAKE_REF": str(target_flake_ref),
        "TARGET_SYSTEM": str(target_system),
    }
    logger.info(
        "extra_env=%s",
        " ".join(f"{k}={v}" for k, v in extra_env.items()),
    )
    finder_env = os.environ.copy()
    for k, v in extra_env.items():
        finder_env[k] = v

    # start process to find derivations directly available
    finder_process = subprocess.Popen(
        args=[
            arg
            for arg in [
                "nix",
                "--offline" if offline else None,
                "eval",
                "--extra-experimental-features",
                "nix-command flakes",
                "--json",
                "--file",
                str(pathlib.Path(__file__).parent.joinpath("find-attribute-paths.nix")),
            ]
            if arg is not None
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=finder_env,
    )
    assert finder_process.stderr is not None

    # while we find these derivations directly available, we process found derivations
    # at the same time to describe them, but also to find derivations indirectly
    # available via their build inputs
    derivation_description_pool = multiprocessing.pool.ThreadPool(n_workers)

    # we can't access the pool queue so we use our own (thanks encapsulation U_U)
    derivation_description_queue = multiprocessing.Queue()
    # we don't want to process the same derivation twice, so we use their output path to
    # check that
    queued_output_paths: set[str] = set()
    visited_output_paths: set[str] = set()

    # we need to use a mutex lock to make sure we write one line at a time
    outfile_lock = Lock()

    # read derivations found by the finder to feed the processing queue
    reader_thread = finder_output_reader(
        finder_process.stderr,
        derivation_description_queue,
        queued_output_paths,
        logger,
    )
    reader_thread.start()

    # process derivations pushed to the processing queue
    process_queue_thread = queue_processor(
        outfile,
        outfile_lock,
        finder_process,
        derivation_description_pool,
        derivation_description_queue,
        queued_output_paths,
        visited_output_paths,
        finder_env,
        offline,
        logger,
    )
    process_queue_thread.start()

    # all is done once finder is done, reader is done and processing queue is done
    finder_process.wait()
    logger.info("FINDER EXIT")
    reader_thread.join()
    logger.info("READER THREAD EXIT")
    process_queue_thread.join()
    logger.info("PROCESS QUEUE THREAD EXIT")
    derivation_description_pool.close()
    derivation_description_pool.join()
    logger.info("POOL EXIT")

    if not derivation_description_queue.empty():
        logger.error("Finished but queue is not empty")
        sys.exit(1)
