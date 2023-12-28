from collections import deque
import logging
from typing import IO, NewType


import nix
import nix.util
from nix.expr import Type, Value

from nixtract.c_bindings.deque import maybe_popleft
from nixtract.model import Derivation

AttributePath = NewType("AttributePath", list[str])

_LOGGER = logging.getLogger(__name__)


def join_attribute_path(attribute_path: AttributePath) -> str:
    return ".".join(map(str, attribute_path))


def maybe_get_attr(
    value: Value,
    key: str | int,
) -> Value | None:
    """
    Get an attribute from a value if it exists and evaluates without error.
    """
    if value.get_type() == Type.list:
        key = int(key)

    try:
        return value[key]
    except (
        nix.util.ThrownError,
        nix.util.AssertionError,
        nix.util.NixError,
    ):
        return None


def value_at_attribute_path(
    root_value: Value,
    attribute_path: AttributePath,
) -> Value | None:
    """
    Get the value at an attribute path if it exists and evaluates without error.
    """
    value = root_value
    for key in attribute_path:
        value = maybe_get_attr(value, key)
        if value is None:
            return None
    return value


def describe_derivation(
    attribute_path: AttributePath, drv_path, outfile: IO[str], value
):
    # try to get the output path
    # may raise evaluation errors
    maybe_out_path = maybe_get_attr(value, "outPath")
    if maybe_out_path is None:
        return
    out_path = maybe_out_path.force()
    assert isinstance(out_path, str)

    # describe the derivation
    name = value["name"].force()
    assert isinstance(name, str)

    derivation = Derivation(
        attribute_path=join_attribute_path(attribute_path),
        derivation_path=drv_path,
        output_path=out_path,
        name=name,
        # TODO
        outputs=[],
        build_inputs=[],
    )
    outfile.write(derivation.json() + "\n")
    outfile.flush()


def visit(
    outfile: IO[str],
    visit_queue: deque[AttributePath],
    visited_drv_paths: set[str],
    root_value: Value,
    attribute_path: AttributePath,
):
    # get value
    value = value_at_attribute_path(root_value, attribute_path)
    assert value is not None

    # if value is a list, add items to the queue
    if value.get_type() == Type.list:
        for i, item_value in enumerate(value):
            item_attribute_path = attribute_path + [str(i)]
            visit_queue.append(AttributePath(item_attribute_path))
        return

    if value.get_type() == Type.attrs:
        # if value is a derivation, describe it and add its inputs to the queue.
        if "type" in value and value["type"].force() == "derivation":
            # check if we've already seen this derivation
            drv_path = value["drvPath"].force()
            assert isinstance(drv_path, str)
            if drv_path in visited_drv_paths:
                return
            visited_drv_paths.add(drv_path)

            # TODO: this is what makes the memory go wild, need to improve
            # describe_derivation(attribute_path, drv_path, outfile, value)

            # add the derivation's inputs to the queue
            if "buildInputs" in value:
                for i, item_value in enumerate(value["buildInputs"]):
                    item_attribute_path = attribute_path + [str(i)]
                    visit_queue.append(AttributePath(item_attribute_path))
            return

        # if value is an attribute set, add its attributes to the queue
        for attr_name in value.keys():
            _LOGGER.debug("attr_name: %s", attr_name)
            item_attribute_path = attribute_path + [attr_name]
            item_value = maybe_get_attr(value, attr_name)
            if item_value is not None:
                visit_queue.append(AttributePath(item_attribute_path))
        return

    # try to limit memory footprint
    del value


def extract(
    outfile: IO[str],
    target_flake_ref: str,
    target_system: str,
    target_attribute_path: str,
):
    """
    Extract the graph of derivations from a flake as JSONL to the output path.
    """
    # reach the root attribute set
    logging.debug("Evaluating flake")
    builtins = nix.eval("builtins")
    get_flake = builtins["getFlake"]

    flake = get_flake(target_flake_ref)
    flake_outputs = flake["outputs"]

    flake_outputs_packages = (
        flake_outputs["packages"]
        if "packages" in flake_outputs
        else flake_outputs["legacyPackages"]
    )

    if target_system not in flake_outputs_packages:
        raise ValueError(
            f"Target system {target_system} not found in flake outputs: {flake_outputs}"
        )

    root_value = flake_outputs_packages[target_system]
    if target_attribute_path:
        parsed_target_attribute_path = AttributePath(target_attribute_path.split("."))
        root_value = value_at_attribute_path(root_value, parsed_target_attribute_path)
        if root_value is None:
            raise ValueError(
                f"Target attribute path {target_attribute_path} not found in flake outputs: {flake_outputs}"
            )

    # start the extraction
    visit_queue: deque[AttributePath] = deque()
    visited_drv_paths: set[str] = set()

    visit_queue.append(AttributePath([]))

    count = 0
    while (attribute_path := maybe_popleft(visit_queue)) is not None:
        count += 1
        _LOGGER.debug(
            "%s/%s %s", count, len(visit_queue), ".".join(map(str, attribute_path))
        )
        visit(
            outfile=outfile,
            visit_queue=visit_queue,
            visited_drv_paths=visited_drv_paths,
            root_value=root_value,
            attribute_path=attribute_path,
        )
