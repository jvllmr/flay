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
    "functools.wraps",
    "typing.no_type_check",
    "typing.overload",
}

_ecosystem_decorators: set[str] = {"pydantic.v1.main.dataclass_transform"}


@cache
def get_default_safe_decorators() -> set[str]:
    return {*_builtin_decorators, *_stdlib_decorators, *_ecosystem_decorators}
