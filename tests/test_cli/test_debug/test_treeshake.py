from pydantic_settings import CliApp
from flay.cli.app import Flay


def test_cli_debug_bundle_then_treeshake_package() -> None:
    CliApp.run(
        Flay,
        [
            "debug-app",
            "treeshake",
            "bundle-then-treeshake-package",
            "rich",
        ],
    )
