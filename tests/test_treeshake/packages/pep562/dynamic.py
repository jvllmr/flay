# example from pydantic
import typing as t
import importlib
_dynamic_names = {"BaseModel": (__spec__.parent, ".main")}


def __getattr__(name: str) -> t.Any:
    if name in _dynamic_names:
        package, module_name = _dynamic_names[name]
        module = importlib.import_module(module_name,package=package)
        return getattr(module, name)

    raise AttributeError("Not here")


def __dir__() -> list[str]:
    return list(_dynamic_names)
