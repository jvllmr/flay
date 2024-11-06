import typer

from flay.common.logging import enable_debug_logging
from .bundle import debug_bundle_app
from .treeshake import debug_treeshake_app

debug_app = typer.Typer()
debug_app.add_typer(debug_bundle_app, name="bundle")
debug_app.add_typer(debug_treeshake_app, name="treeshake")


@debug_app.callback()
def debug_app_main() -> None:
    enable_debug_logging()
