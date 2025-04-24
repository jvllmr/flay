from flay.bundle import DEFAULT_BUNDLE_METADATA, DEFAULT_VENDOR_MODULE_NAME
from flay.common.logging import enable_debug_logging
from .debug import DebugApp
from flay.common.pydantic import FlayBaseSettings
from pydantic_settings import CliApp, CliSubCommand, SettingsError, CliPositionalArg
from .options import DebugFlagT
import os
from pydantic import Field, AliasChoices
from pathlib import Path
import typing as t
from .bundle import cli_bundle_package
from .treeshake import cli_treeshake_package
from flay.common.rich import console, check


class Flay(FlayBaseSettings):
    if not os.getenv("FLAY_DEBUG_APP"):
        module_spec: CliPositionalArg[
            t.Annotated[str, Field(description="Module that should be bundled")]
        ]
    output_path: t.Annotated[
        Path,
        Field(
            description="Target path for the generated bundle",
            validation_alias=AliasChoices("output", "o"),
        ),
    ] = Path("flayed")
    vendor_module_name: t.Annotated[
        str, Field(description="Name of the module where external packages should be")
    ] = DEFAULT_VENDOR_MODULE_NAME
    bundle_metadata: t.Annotated[
        bool,
        Field(
            description="Whether package metadata should be collocated with the generated bundle"
        ),
    ] = DEFAULT_BUNDLE_METADATA

    treeshake: t.Annotated[
        bool,
        Field(
            description="Should unused source code be stripped from the bundle?",
            validation_alias=AliasChoices("t"),
        ),
    ] = True

    debug: DebugFlagT = False
    if os.getenv("FLAY_DEBUG_APP"):
        debug_app: CliSubCommand[DebugApp]

    def main(self) -> None:
        console.print(f"Starting to bundle module {self.module_spec}...")
        cli_bundle_package(
            self.module_spec,
            self.output_path,
            self.vendor_module_name,
            self.bundle_metadata,
        )
        console.print(check, f"Finished bundling {self.module_spec}")
        if self.treeshake:
            console.print("Start removing unused code...")
            cli_treeshake_package(str(self.output_path.absolute()))
            console.print(check, "Finished removing unused code")

    def cli_cmd(self) -> None:
        if self.debug:
            enable_debug_logging()  # pragma: no cover
        try:
            CliApp.run_subcommand(self, False)
        except SettingsError:
            self.main()
