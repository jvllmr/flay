from flay._flay_rs import FileCollector
import typing as t

from flay.common.pydantic import FlayBaseSettings, config_from_pydantic
from flay.bundle.package import bundle_package
from ...common.module_spec import find_all_files_in_module_spec
import logging
from pathlib import Path
import shutil
from pydantic import Field, DirectoryPath
import click

log = logging.getLogger(__name__)


@click.group()
def debug_bundle_app() -> None:
    pass


debug_bundle_app.name = "bundle"


class DebugBundleConfig(FlayBaseSettings):
    module_spec: t.Annotated[str, Field(description="A module spec to test")]
    dest_path: t.Annotated[
        DirectoryPath,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed, treeshaked bundle",
        ),
    ]


@debug_bundle_app.command("collector")
@config_from_pydantic(DebugBundleConfig)
def bundle_collector(config: DebugBundleConfig) -> None:
    collector = FileCollector(package=config.module_spec)
    for path in find_all_files_in_module_spec(config.module_spec):
        log.debug("Found: %s", path)
        file_module_spec = (
            config.module_spec
            if path.name == "__init__.py"
            else f"{config.module_spec}.{path.stem}"
        )
        collector._process_module(file_module_spec)

    print({str(k): type(v) for k, v in collector.collected_files.items()})  # noqa: T201


class DebugBundlePackageConfig(DebugBundleConfig):
    dest_path: t.Annotated[
        Path,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed bundle",
        ),
    ]


@debug_bundle_app.command("bundle_package")
@config_from_pydantic(DebugBundlePackageConfig)
def debug_bundle_package(config: DebugBundlePackageConfig) -> None:
    if config.dest_path.exists():
        shutil.rmtree(str(config.dest_path))
    bundle_package(module_spec=config.module_spec, destination_path=config.dest_path)
