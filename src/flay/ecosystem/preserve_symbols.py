from functools import cache
from pydantic import _dynamic_imports as pydantic_dynamic_imports


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

    return res


def enrich_preserve_symbols_from_import_aliases(
    symbols: set[str], import_aliases: dict[str, str]
) -> None:
    # while loop for catching transitive relationships
    new_added = True
    while new_added:
        new_added = False
        for k, v in import_aliases.items():
            if k in symbols and v not in symbols:
                symbols.add(v)
                new_added = True
            elif v in symbols and k not in symbols:
                symbols.add(k)
                new_added = True
