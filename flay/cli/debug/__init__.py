import typer

from flay.common.logging import enable_debug_logging
from .bundle import debug_bundle_app

debug_app = typer.Typer()
debug_app.add_typer(debug_bundle_app, name="bundle")


@debug_app.callback()
def debug_app_main() -> None:
    enable_debug_logging()
