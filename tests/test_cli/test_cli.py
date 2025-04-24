from flay.cli.app import Flay
from pydantic_settings import CliApp
from pathlib import Path


def test_cli_bundle_treeshake_flay() -> None:
    CliApp.run(Flay, ["bundle", "flay"])


def test_cli_bundle_treeshake_flay_with_metadata() -> None:
    CliApp.run(Flay, ["bundle", "flay", "--bundle-metadata", "True"])
    result_path = Path("flayed")
    assert result_path.exists()
    assert len(list(result_path.glob("flay-*.dist-info"))) > 0
