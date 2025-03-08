from flay._flay_rs import FileCollector
import typing as t
from flay.cli.debug.types import DebugModuleSpecT
from flay.bundle.package import bundle_package
from ...common.module_spec import find_all_files_in_module_spec
import logging
from pathlib import Path
import shutil
from pydantic import BaseModel, ConfigDict, Field
from pydantic_settings import CliSubCommand, CliApp


log = logging.getLogger(__name__)


class DebugBundleCollectorCmd(BaseModel):
    module_spec: DebugModuleSpecT

    def cli_cmd(self) -> None:
        collector = FileCollector(package=self.module_spec)
        for path in find_all_files_in_module_spec(self.module_spec):
            log.debug("Found: %s", path)
            file_module_spec = (
                self.module_spec
                if path.name == "__init__.py"
                else f"{self.module_spec}.{path.stem}"
            )
            collector._process_module(file_module_spec)

        print({str(k): type(v) for k, v in collector.collected_files.items()})  # noqa: T201


class DebugBundlePackageCmd(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    module_spec: DebugModuleSpecT
    path: t.Annotated[
        Path,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed bundle",
        ),
    ]

    def cli_cmd(self) -> None:
        if self.path.exists():
            shutil.rmtree(str(self.path))
        bundle_package(module_spec=self.module_spec, destination_path=self.path)


class DebugBundleApp(BaseModel):
    collector: CliSubCommand[DebugBundleCollectorCmd]
    bundle_package: CliSubCommand[DebugBundlePackageCmd]

    def cli_cmd(self) -> None:
        CliApp.run_subcommand(self)
