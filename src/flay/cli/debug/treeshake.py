import logging
from pathlib import Path

import typing as t


from flay.common.logging import enable_debug_logging
from flay.common.pydantic import FlayBaseSettings
from flay.treeshake.package import treeshake_package
from .types import DebugModuleSpecT
from pydantic import Field
from pydantic import DirectoryPath
from click.testing import CliRunner
from clonf import clonf_click
import click


log = logging.getLogger(__name__)


@click.group(name="treeshake")
def debug_treeshake_app() -> None:
    enable_debug_logging()


class DebugTreeshakeBundleThenTreeshakeSettings(FlayBaseSettings):
    module_spec: DebugModuleSpecT
    path: t.Annotated[
        DirectoryPath,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed, treeshaked bundle",
        ),
    ]


@debug_treeshake_app.command(name="bundle_then_treeshake_package")
@clonf_click
def bundle_then_treeshake_cmd(
    settings: DebugTreeshakeBundleThenTreeshakeSettings,
) -> None:
    from .bundle import debug_bundle_package_cmd

    runner = CliRunner()
    runner.invoke(
        debug_bundle_package_cmd, [settings.module_spec, "--path", str(settings.path)]
    )
    stats = treeshake_package(str(settings.path))
    print("Removed statements count:", stats)  # noqa: T201
