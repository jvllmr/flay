from flay.cli.app import flay
from click.testing import CliRunner

runner = CliRunner()


def test_cli_debug_bundle_then_treeshake_package() -> None:
    runner.invoke(
        flay,
        [
            "debug",
            "treeshake",
            "bundle_then_treeshake_package",
            "rich",
        ],
    )
