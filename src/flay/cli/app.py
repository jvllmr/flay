from flay.bundle import DEFAULT_BUNDLE_METADATA, DEFAULT_VENDOR_MODULE_NAME
from flay.common.logging import enable_debug_logging

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
    debug: t.Annotated[bool, CliOption(), Field(description="Enable debug logging")] = (
        False
    )


@click.group()
@clonf_click
def flay(debug_setting: DebugSetting) -> None:
    if debug_setting.debug:
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
        ),
    ] = DEFAULT_VENDOR_MODULE_NAME
    bundle_metadata: t.Annotated[
        bool,
        CliOption(),
        Field(
            description="Whether package metadata should be collocated with the generated bundle",
            alias="bundle-metadata",
        ),
    ] = DEFAULT_BUNDLE_METADATA

    treeshake: t.Annotated[
        bool,
        CliOption(),
        Field(
            description="Should unused source code be stripped from the bundle?",
        ),
    ] = True


@flay.command(name="bundle")
@clonf_click
def flay_main(settings: FlayMainSettings) -> None:
    console.print(f"Starting to bundle module {settings.module_spec}...")
    cli_bundle_package(
        settings.module_spec,
        settings.output_path,
        settings.vendor_module_name,
        settings.bundle_metadata,
    )
    console.print(check, f"Finished bundling {settings.module_spec}")
    if settings.treeshake:
        console.print("Start removing unused code...")
        cli_treeshake_package(str(settings.output_path.absolute()))
        console.print(check, "Finished removing unused code")


if os.getenv("FLAY_DEBUG_APP"):
    flay.add_command(debug_app)
