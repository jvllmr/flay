from .. import runner
from flay.cli.app import app


def test_cli_debug_bundle_then_treeshake_package() -> None:
    runner.invoke(
        app,
        [
            "debug",
            "treeshake",
            "bundle_then_treeshake_package",
            "--module-spec",
            "click",
        ],
    )
