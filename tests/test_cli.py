import json
import logging
from io import StringIO
from pathlib import Path

from click.testing import CliRunner


def test_trivial():
    import nixtract.cli
    from nixtract.cli import cli

    # need absolute path
    flake_path = (Path(__file__).parent / "fixtures" / "flake-trivial").absolute()
    # sanity checks
    assert flake_path.exists()
    assert flake_path.is_dir()
    assert (flake_path / "flake.nix").exists()
    assert (flake_path / "flake.lock").exists()

    # CLI logger
    cli_stderr_handler = StringIO()
    logging.getLogger(nixtract.cli.__name__).addHandler(
        logging.StreamHandler(stream=cli_stderr_handler)
    )

    # test CLI
    runner = CliRunner(
        mix_stderr=False,
    )
    result = runner.invoke(
        cli=cli,
        args=[
            "--target-flake-ref",
            f"path:{flake_path}",
            "--target-system",
            "x86_64-linux",
            "--verbose",
            # print to stdout
            "-",
        ],
    )

    # test success
    assert result.exit_code == 0

    # test output node
    # trivial flake should contain a single node without dependencies
    assert result.stdout != "", "Empty result"
    result_node = json.loads(result.stdout)
    assert result_node.get("output_path") is not None
    assert result_node.get("name") == "trivial-1.0"
    assert result_node.get("parsed_name", {}).get("name") == "trivial"
    assert result_node.get("parsed_name", {}).get("version") == "1.0"
    assert result_node.get("build_inputs") == []

    # test behavior through logs
    # quite unstable, will do for now
    cli_stderr_handler.seek(0)
    cli_stderr = cli_stderr_handler.read()
    assert "FINDER EXIT" in cli_stderr
    assert "QUEUE PROCESSOR CLOSED" in cli_stderr
    assert "READER THREAD EXIT" in cli_stderr
    assert "PROCESS QUEUE THREAD EXIT" in cli_stderr
    assert "POOL EXIT" in cli_stderr


def test_direct_buildinput():
    import nixtract.cli
    from nixtract.cli import cli

    # need absolute path
    flake_path = (
        Path(__file__).parent / "fixtures" / "flake-direct-buildInput"
    ).absolute()
    # sanity checks
    assert flake_path.exists()
    assert flake_path.is_dir()
    assert (flake_path / "flake.nix").exists()
    assert (flake_path / "flake.lock").exists()

    # CLI logger
    cli_stderr_handler = StringIO()
    logging.getLogger(nixtract.cli.__name__).addHandler(
        logging.StreamHandler(stream=cli_stderr_handler)
    )

    # test CLI
    runner = CliRunner(
        mix_stderr=False,
    )
    result = runner.invoke(
        cli=cli,
        args=[
            "--target-flake-ref",
            f"path:{flake_path}",
            "--target-system",
            "x86_64-linux",
            "--verbose",
            # print to stdout
            "-",
        ],
    )

    # test success
    assert result.exit_code == 0

    # test output node
    # this flake should contain at least 2 nodes, pkg1 and pkg2
    # pkg2 should have pkg1 as a build input
    assert result.stdout != "", "Empty result"
    stdout_lines = result.stdout.splitlines()
    assert len(stdout_lines) >= 2
    result_nodes = [json.loads(line) for line in stdout_lines]
    pkg2_node = next((node for node in result_nodes if node["name"] == "pkg2"), None)
    assert pkg2_node is not None
    assert pkg2_node.get("output_path") is not None
    assert pkg2_node.get("name") == "pkg2"
    assert pkg2_node.get("parsed_name", {}).get("name") == "pkg2"
    assert len(pkg2_node.get("build_inputs")) >= 1
    pkg2_buildinput_pkg1 = next(
        (
            bi
            for bi in pkg2_node.get("build_inputs")
            if bi["output_path"].endswith("-pkg1")
        ),
        None,
    )
    assert pkg2_buildinput_pkg1 is not None

    # make sure that the build input has been described as well
    pkg1_node = next((node for node in result_nodes if node["name"] == "pkg1"), None)
    assert pkg1_node is not None
    assert pkg1_node.get("output_path") == pkg2_buildinput_pkg1["output_path"]
    assert pkg1_node.get("output_path") == pkg2_buildinput_pkg1["output_path"]

    # test behavior through logs
    # quite unstable, will do for now
    cli_stderr_handler.seek(0)
    cli_stderr = cli_stderr_handler.read()
    assert "FINDER EXIT" in cli_stderr
    assert "QUEUE PROCESSOR CLOSED" in cli_stderr
    assert "READER THREAD EXIT" in cli_stderr
    assert "PROCESS QUEUE THREAD EXIT" in cli_stderr
    assert "POOL EXIT" in cli_stderr
