from pydantic_settings import CliPositionalArg
import typing_extensions as te
from pydantic import Field
import typing as t

DebugModuleSpecT: te.TypeAlias = CliPositionalArg[
    t.Annotated[str, Field(description="A module spec to test")]
]
