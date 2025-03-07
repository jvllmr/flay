import click
from flay.common.logging import enable_debug_logging
from .bundle import debug_bundle_app
from .treeshake import debug_treeshake_app


@click.group()
def debug_app() -> None:
    enable_debug_logging()


debug_app.name = "debug"

debug_app.add_command(debug_bundle_app)
debug_app.add_command(debug_treeshake_app)
