from flay.cli.app import flay
from click.testing import CliRunner
from pathlib import Path
import os
from importlib.metadata import Distribution

runner = CliRunner()


def test_cli_bundle_treeshake_flay() -> None:
    runner.invoke(flay, ["bundle", "flay"])


def test_cli_bundle_treeshake_flay_with_metadata() -> None:
    runner.invoke(flay, ["bundle", "flay", "--bundle-metadata", "True"])
    result_path = Path("flayed")
    dist = Distribution.from_name("flay")
    assert result_path.exists()
    assert f"flay-{dist.version}.dist-info" in os.listdir(result_path)
