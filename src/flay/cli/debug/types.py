import typing_extensions as te
from pydantic import Field
import typing as t
from clonf import CliArgument

DebugModuleSpecT: te.TypeAlias = t.Annotated[
    str, CliArgument(), Field(description="A module spec to test")
]
