from flay.bundle import DEFAULT_BUNDLE_METADATA, DEFAULT_VENDOR_MODULE_NAME
from flay.common.logging import enable_debug_logging

from flay.common.module_spec import get_top_level_package
from flay.common.pydantic import FlayBaseSettings
from pydantic_settings import CliPositionalArg

import os
from pydantic import Field, AliasChoices
from pathlib import Path
import typing as t
from .bundle import cli_bundle_package
from .treeshake import cli_treeshake_package
from flay.common.rich import console, check
from .debug import debug_app
import click
from clonf.integrations.click import clonf_click
from clonf import CliArgument, CliOption


class DebugSetting(FlayBaseSettings):
    debug: t.Annotated[
        bool, CliOption(is_flag=True), Field(description="Enable debug logging")
    ] = False


@click.group()
@clonf_click
def flay(debug_setting: DebugSetting) -> None:
    if debug_setting.debug:  # pragma: no cover
        enable_debug_logging()


class FlayMainSettings(FlayBaseSettings):
    module_spec: CliPositionalArg[
        t.Annotated[
            str,
            CliArgument(),
            Field(description="Module that should be bundled"),
        ]
    ]
    output_path: t.Annotated[
        Path,
        CliOption(),
        Field(
            description="Target path for the generated bundle",
            alias="output-path",
            validation_alias=AliasChoices("output", "o"),
        ),
    ] = Path("flayed")
    vendor_module_name: t.Annotated[
        str,
        CliOption(),
        Field(
            description="Name of the module where external packages should be",
            alias="vendor-module-name",
            validation_alias=AliasChoices("vendor-module-name", "vendor_module_name"),
        ),
    ] = DEFAULT_VENDOR_MODULE_NAME
    bundle_metadata: t.Annotated[
        bool,
        CliOption(is_flag=True),
        Field(
            description="Whether package metadata should be collocated with the generated bundle",
            alias="bundle-metadata/--no-bundle-metadata",
            validation_alias=AliasChoices("bundle-metadata", "bundle_metadata"),
        ),
    ] = DEFAULT_BUNDLE_METADATA

    treeshake: t.Annotated[
        bool,
        CliOption(),
        Field(
            description="Should unused source code be stripped from the bundle?",
        ),
    ] = True
    resources: t.Annotated[
        dict[str, str],
        CliOption(),
        Field(
            description="Resources that should be bundled. Accepts a module spec mapped to a glob pattern",
            default_factory=dict,
        ),
    ]


@flay.command(name="bundle")
@clonf_click
def flay_main(settings: FlayMainSettings) -> None:
    console.print(f"Starting to bundle module {settings.module_spec}...")
    cli_bundle_package(
        settings.module_spec,
        settings.output_path,
        settings.vendor_module_name,
        settings.bundle_metadata,
        settings.resources,
    )
    console.print(check, f"Finished bundling {settings.module_spec}")
    if settings.treeshake:
        console.print("Start removing unused code...")
        vendor_prefix = f"{get_top_level_package(settings.module_spec)}.{settings.vendor_module_name}"
        removed_stmts_count = cli_treeshake_package(
            str(settings.output_path.absolute()), vendor_prefix=vendor_prefix
        )
        console.print(
            check,
            f"Finished removing unused code. Removed {removed_stmts_count} statements in total",
        )


if os.getenv("FLAY_DEBUG_APP"):
    flay.add_command(debug_app)
