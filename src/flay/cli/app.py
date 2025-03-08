from .debug import DebugApp
from flay.common.pydantic import FlayBaseSettings
from pydantic_settings import CliApp, CliSubCommand


class Flay(FlayBaseSettings):
    debug: CliSubCommand[DebugApp]

    def cli_cmd(self) -> None:
        CliApp.run_subcommand(self)
