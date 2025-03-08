from flay.common.logging import enable_debug_logging
from .debug import DebugApp
from flay.common.pydantic import FlayBaseSettings
from pydantic_settings import CliApp, CliSubCommand
from .options import DebugFlagT
import os


class Flay(FlayBaseSettings):
    debug: DebugFlagT = False
    if os.getenv("FLAY_DEBUG_APP"):
        debug_app: CliSubCommand[DebugApp]

    def cli_cmd(self) -> None:
        if self.debug:
            enable_debug_logging()
        CliApp.run_subcommand(self)
