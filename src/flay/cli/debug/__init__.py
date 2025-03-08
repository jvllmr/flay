from flay.common.logging import enable_debug_logging
from .bundle import DebugBundleApp
from .treeshake import DebugTreeshakeApp
from pydantic import BaseModel
from pydantic_settings import CliApp, CliSubCommand


class DebugApp(BaseModel):
    bundle: CliSubCommand[DebugBundleApp]
    treeshake: CliSubCommand[DebugTreeshakeApp]

    def cli_cmd(self) -> None:
        enable_debug_logging()
        CliApp.run_subcommand(self)
