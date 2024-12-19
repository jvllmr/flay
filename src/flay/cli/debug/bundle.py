import typer
from flay._flay_rs.bundle import FileCollector
import typing as t

from flay.bundle.package import bundle_package
from ...common.module_spec import find_all_files_in_module_spec
import logging
from pathlib import Path
import shutil

debug_bundle_app = typer.Typer()

log = logging.getLogger(__name__)


@debug_bundle_app.command("collector")
def bundle_collector(
    module_spec: t.Annotated[str, typer.Argument(help="A module spec to test")],
) -> None:
    collector = FileCollector(package=module_spec)
    for path in find_all_files_in_module_spec(module_spec):
        log.debug(f"Found: {path}")
        file_module_spec = (
            module_spec
            if path.name == "__init__.py"
            else f"{module_spec}.{path.name}"[:-3]
        )
        collector._process_module(file_module_spec)

    typer.echo({str(k): type(v) for k, v in collector.collected_files.items()})


@debug_bundle_app.command("bundle_package")
def debug_bundle_package(
    module_spec: t.Annotated[str, typer.Argument(help="A module spec to test")],
    dest_path: t.Annotated[
        Path,
        typer.Argument(
            default_factory=lambda: Path("./debug_bundle"),
            help="Destination path for the completed bundle",
            resolve_path=True,
        ),
    ],
) -> None:
    if dest_path.exists():
        shutil.rmtree(str(dest_path))
    bundle_package(module_spec=module_spec, destination_path=dest_path)
