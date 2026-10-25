from functools import cache


_annotation_using_ecosystems: set[str] = {
    "attr.attrs",
    "attr.attrib",
    "attr.s",
    "attrs.define",
    "attrs.frozen",
    "attrs.mutable",
    "dataclasses.dataclass",
    "msgspec.Struct",
    "pydantic.BaseModel",
    "pydantic.v1.BaseModel",
    "typing.NamedTuple",
    "typing.Protocol",
    "typing.TypedDict",
    "typing_extensions.NamedTuple",
    "typing_extensions.Protocol",
    "typing_extensions.TypedDict",
}


@cache
def get_default_annotation_ecosystems() -> set[str]:
    """Decorators and base classes of ecosystems that read type annotations
    at runtime. Annotations on these classes must not be removed."""
    return set(_annotation_using_ecosystems)
