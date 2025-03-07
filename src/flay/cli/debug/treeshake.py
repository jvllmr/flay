import logging
from pathlib import Path

import typing as t
from .bundle import debug_bundle_package, DebugBundlePackageConfig
from flay.treeshake.package import treeshake_package
from flay.common.pydantic import config_from_pydantic, FlayBaseSettings
from pydantic import Field
from pydantic import DirectoryPath

log = logging.getLogger(__name__)

import click


@click.group()
def debug_treeshake_app() -> None:
    pass


debug_treeshake_app.name = "treeshake"


class DebugBundleThenTreeshakeConfig(FlayBaseSettings):
    module_spec: t.Annotated[str, Field(description="A module spec to test")]
    dest_path: t.Annotated[
        DirectoryPath,
        Field(
            default_factory=lambda: Path("./debug_bundle"),
            description="Destination path for the completed, treeshaked bundle",
        ),
    ]


@debug_treeshake_app.command("bundle_then_treeshake_package")
@config_from_pydantic(DebugBundleThenTreeshakeConfig)
def debug_bundle_then_treeshake_package(config: DebugBundleThenTreeshakeConfig) -> None:
    new_ctx = click.Context(debug_bundle_package)
    new_ctx.params["config"] = DebugBundlePackageConfig.model_validate(config)
    debug_bundle_package.invoke(
        ctx=new_ctx,
    )
    stats = treeshake_package(str(config.dest_path))
    print(dict(stats))  # noqa: T201
