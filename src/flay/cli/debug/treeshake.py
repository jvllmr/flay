import logging
from pathlib import Path

import typing as t

from pydantic_settings import CliApp, CliSubCommand

from flay.cli.debug.types import DebugModuleSpecT

from flay.treeshake.package import treeshake_package

from pydantic import Field
from pydantic import DirectoryPath
from pydantic import BaseModel

log = logging.getLogger(__name__)


class DebugTreeshakeBundleThenTreeshakeCmd(BaseModel):
    module_spec: DebugModuleSpecT
    path: t.Annotated[
        DirectoryPath,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed, treeshaked bundle",
        ),
    ]

    def cli_cmd(self) -> None:
        from .bundle import DebugBundlePackageCmd

        DebugBundlePackageCmd.model_validate(self).cli_cmd()
        stats = treeshake_package(str(self.path))
        print(dict(stats))  # noqa: T201


class DebugTreeshakeApp(BaseModel):
    bundle_then_treeshake_package: CliSubCommand[DebugTreeshakeBundleThenTreeshakeCmd]

    def cli_cmd(self) -> None:
        CliApp.run_subcommand(self)
