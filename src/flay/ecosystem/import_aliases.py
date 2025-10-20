from functools import cache
from pydantic import _dynamic_imports as pydantic_dynamic_imports
from clonf import _dynamic_imports as clonf_dynamic_imports


@cache
def get_import_aliases() -> dict[str, str]:
    res: dict[str, str] = {}

    for obj, (parent, module) in pydantic_dynamic_imports.items():
        if module == "__module__":
            continue
        else:
            res[f"pydantic.{obj}"] = f"{parent}{module}.{obj}"

    for obj, module in clonf_dynamic_imports.items():
        res[f"clonf.{obj}"] = f"{module}.{obj}"

    return res
