from click.testing import CliRunner
from flay.cli.app import flay

runner = CliRunner()


def test_cli_debug_bundle_collector() -> None:
    runner.invoke(flay, ["debug", "bundle", "collector", "rich"])


def test_cli_debug_bundle_package() -> None:
    runner.invoke(flay, ["debug", "bundle", "bundle_package", "rich"])
