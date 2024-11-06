import logging
from pathlib import Path
import typer
import typing as t
from .bundle import debug_bundle_package
from flay.treeshake.package import treeshake_package

debug_treeshake_app = typer.Typer()

log = logging.getLogger(__name__)


@debug_treeshake_app.command("bundle_then_treeshake_package")
def debug_bundle_then_treeshake_package(
    module_spec: t.Annotated[str, typer.Argument(help="A module spec to test")],
    dest_path: t.Annotated[
        Path,
        typer.Argument(
            default_factory=lambda: Path("./debug_bundle"),
            help="Destination path for the completed, treeshaked bundle",
            resolve_path=True,
        ),
    ],
) -> None:
    debug_bundle_package(module_spec, dest_path)
    treeshake_package(str(dest_path))
