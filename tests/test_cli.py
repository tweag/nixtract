import json
import logging
from io import StringIO
from pathlib import Path

from click.testing import CliRunner


def test_trivial():
    import nixtract.cli
    from nixtract.cli import cli

    # need absolute path
    trivial_flake_path = (
        Path(__file__).parent / "fixtures" / "flake-trivial"
    ).absolute()
    # sanity checks
    assert trivial_flake_path.exists()
    assert trivial_flake_path.is_dir()
    assert (trivial_flake_path / "flake.nix").exists()
    assert (trivial_flake_path / "flake.lock").exists()

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
            f"path:{trivial_flake_path}",
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
