from flay.cli.app import flay
from click.testing import CliRunner
from pathlib import Path

runner = CliRunner()


def test_cli_bundle_treeshake_flay() -> None:
    runner.invoke(flay, ["bundle", "flay"])


def test_cli_bundle_treeshake_flay_with_metadata() -> None:
    runner.invoke(flay, ["bundle", "flay", "--bundle-metadata", "True"])
    result_path = Path("flayed")
    assert result_path.exists()
    assert len(list(result_path.glob("flay-*.dist-info"))) > 0
