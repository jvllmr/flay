from functools import cache


_builtin_decorators: set[str] = {
    "__builtin__.classmethod",
    "__builtin__.staticmethod",
    "__builtin__.property",
}

_stdlib_decorators: set[str] = {
    "abc.abstractmethod",
    "contextlib.asynccontextmanager",
    "contextlib.contextmanager",
    "dataclasses.dataclass",
    "functools.cache",
    "functools.cached_property",
    "functools.lru_cache",
    "functools.total_ordering",
    "functools.wraps",
    "typing.dataclass_transform",
    "typing.final",
    "typing.no_type_check",
    "typing.overload",
    "typing_extensions.dataclass_transform",
}

_ecosystem_decorators: set[str] = {
    "attr.attrs",
    "attr._make.attrs",
    "clonf.clonf_click",
    "pydantic.root_validator",
    "pydantic.validator",
    "pydantic.v1.class_validators.root_validator",
    "pydantic.v1.class_validators.validator",
}


@cache
def get_default_safe_decorators() -> set[str]:
    return {*_builtin_decorators, *_stdlib_decorators, *_ecosystem_decorators}
