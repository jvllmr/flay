from functools import cache
from pydantic import _dynamic_imports as pydantic_dynamic_imports
from clonf import _dynamic_imports as clonf_dynamic_imports


@cache
def get_treeshake_fixed_preservations() -> set[str]:
    res: set[str] = set()

    for obj, (parent, module) in pydantic_dynamic_imports.items():
        if module == "__module__":
            continue
        else:
            res.add(f"{parent}{module}.{obj}")

    for obj, module in clonf_dynamic_imports.items():
        res.add(f"{module}.{obj}")

    return res
