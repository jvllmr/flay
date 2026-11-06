import os
from importlib.metadata import Distribution
from pathlib import Path

from click.testing import CliRunner

from flay.cli.app import flay

runner = CliRunner()


def test_cli_bundle_treeshake_flay() -> None:
    runner.invoke(flay, ["bundle", "flay"])


def test_cli_bundle_treeshake_flay_with_metadata() -> None:
    runner.invoke(flay, ["bundle", "flay", "--bundle-metadata", "True"])
    result_path = Path("flayed")
    dist = Distribution.from_name("flay")
    assert result_path.exists()
    assert f"flay-{dist.version}.dist-info" in os.listdir(result_path)


def test_cli_bundle_with_cache_dir(tmp_path: Path) -> None:
    cache_dir = tmp_path / "custom-cache"
    result = runner.invoke(
        flay, ["bundle", "rich", "--no-bundle-metadata", "--cache-dir", str(cache_dir)]
    )
    assert result.exit_code == 0, result.output
    assert cache_dir.is_dir() and any(cache_dir.iterdir())
