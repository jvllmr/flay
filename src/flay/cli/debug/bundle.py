from flay._flay_rs import FileCollector
import typing as t
from flay.common.logging import enable_debug_logging
from flay.cli.debug.types import DebugModuleSpecT
from flay.bundle.package import bundle_package
from flay.common.pydantic import FlayBaseSettings
from ...common.module_spec import find_all_files_in_module_spec
import logging
from pathlib import Path
import shutil
from pydantic import Field
from clonf import clonf_click, CliOption
import click

log = logging.getLogger(__name__)


@click.group(name="bundle")
def debug_bundle_app() -> None:
    enable_debug_logging()


class DebugBundleCollectorSettings(FlayBaseSettings):
    module_spec: DebugModuleSpecT


@debug_bundle_app.command(name="collector")
@clonf_click
def debug_bundle_collector_cmd(settings: DebugBundleCollectorSettings) -> None:
    collector = FileCollector(package=settings.module_spec)
    for path in find_all_files_in_module_spec(settings.module_spec):
        log.debug("Found: %s", path)
        file_module_spec = (
            settings.module_spec
            if path.name == "__init__.py"
            else f"{settings.module_spec}.{path.stem}"
        )
        collector._process_module(file_module_spec)

    print({str(k): type(v) for k, v in collector.collected_files.items()})  # noqa: T201


class DebugBundlePackageSettings(FlayBaseSettings):
    module_spec: DebugModuleSpecT
    path: t.Annotated[
        Path,
        CliOption(),
        click.Path(file_okay=False, writable=True, path_type=Path),
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed bundle",
        ),
    ]


@debug_bundle_app.command(name="bundle_package")
@clonf_click
def debug_bundle_package_cmd(settings: DebugBundlePackageSettings) -> None:
    if settings.path.exists():
        shutil.rmtree(str(settings.path))
    bundle_package(module_spec=settings.module_spec, destination_path=settings.path)
