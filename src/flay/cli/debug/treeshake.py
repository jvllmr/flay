import logging
from pathlib import Path

import typing as t

from pydantic_settings import CliApp, CliSubCommand


from flay.common.pydantic import FlayBaseModel
from flay.treeshake.package import treeshake_package
from .bundle import DebugBundlePackageCmd
from pydantic import Field
from pydantic import DirectoryPath

log = logging.getLogger(__name__)


class DebugTreeshakeBundleThenTreeshakeCmd(DebugBundlePackageCmd):
    path: t.Annotated[
        DirectoryPath,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed, treeshaked bundle",
        ),
    ]

    def cli_cmd(self) -> None:
        DebugBundlePackageCmd.cli_cmd(DebugBundlePackageCmd.model_validate(self))
        stats = treeshake_package(str(self.path))
        print(dict(stats))  # noqa: T201


class DebugTreeshakeApp(FlayBaseModel):
    bundle_then_treeshake_package: CliSubCommand[DebugTreeshakeBundleThenTreeshakeCmd]

    def cli_cmd(self) -> None:
        CliApp.run_subcommand(self)
