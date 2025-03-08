from flay.common.pydantic import FlayBaseModel
import typing as t
from pydantic import Field
import typing_extensions as te

DebugFlagT: te.TypeAlias = t.Annotated[bool, Field(description="Enable debug logging")]


class DebugOption(FlayBaseModel):
    debug: DebugFlagT = False
