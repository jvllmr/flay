from .. import runner
from flay.cli.app import app


def test_cli_debug_bundle_collector() -> None:
    runner.invoke(app, ["debug", "bundle", "collector", "click"])


def test_cli_debug_bundle_package() -> None:
    runner.invoke(app, ["debug", "bundle", "bundle_package", "click"])
