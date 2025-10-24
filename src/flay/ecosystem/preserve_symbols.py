from functools import cache
from pydantic import _dynamic_imports as pydantic_dynamic_imports
from .import_aliases import get_default_import_aliases


@cache
def get_default_preserve_symbols() -> set[str]:
    res: set[str] = {
        *(),
    }

    for obj, (parent, module) in pydantic_dynamic_imports.items():
        if module == "__module__":
            continue
        else:
            res.add(f"{parent}{module}.{obj}")

    import_aliases = get_default_import_aliases()

    for k, v in import_aliases.items():
        if k in res:
            res.add(v)
        elif v in res:
            res.add(k)

    return res
