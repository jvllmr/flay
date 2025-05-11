from .bundle import debug_bundle_app
from .treeshake import debug_treeshake_app
import click


@click.group(name="debug")
def debug_app() -> None:
    pass


debug_app.add_command(debug_bundle_app)
debug_app.add_command(debug_treeshake_app)
