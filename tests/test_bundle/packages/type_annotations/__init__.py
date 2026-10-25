from dataclasses import dataclass
from typing import NamedTuple, Protocol, TypedDict
import typing as t

annotated_var: list[int] = [1]
bare_annotation: t.Any


def annotated_function(
    first: int,
    second: str = "x",
    *args: int,
    key: bool = False,
    **kwargs: t.Any,
) -> t.Literal["done"]:
    return "done"


@dataclass
class DataclassModel:
    name: str
    value: int = 5


class NamedTupleModel(NamedTuple):
    x: int


class TypedDictModel(TypedDict):
    y: int


class ProtocolModel(Protocol):
    a: int


class PlainClass:
    x: int = 3


def uses_annotations():
    return (
        annotated_function(1),
        annotated_var,
        DataclassModel(name="a").value,
        NamedTupleModel(x=1).x,
    )
