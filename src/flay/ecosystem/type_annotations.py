from functools import cache

_annotation_using_ecosystems: set[str] = {
    "attr.attrs",
    "attr.attrib",
    "attr.s",
    "attrs.define",
    "attrs.frozen",
    "attrs.mutable",
    "clonf.clonf_click",
    "clonf.integrations.click.clonf_click",
    "dataclasses.dataclass",
    "msgspec.Struct",
    "pydantic.BaseModel",
    "pydantic.v1.BaseModel",
    "pydantic_settings.BaseSettings",
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
    at runtime. Annotations on these classes and on functions decorated with
    them must not be removed."""
    return set(_annotation_using_ecosystems)
