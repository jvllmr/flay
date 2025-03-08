from pydantic_settings import CliApp

from flay.cli.app import Flay


def test_cli_debug_bundle_collector() -> None:
    CliApp.run(Flay, ["debug-app", "bundle", "collector", "rich"])


def test_cli_debug_bundle_package() -> None:
    CliApp.run(Flay, ["debug-app", "bundle", "bundle-package", "rich"])
