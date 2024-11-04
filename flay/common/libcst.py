from __future__ import annotations
from importlib.util import resolve_name
from pathlib import Path
from libcst import ImportStar, Module, parse_module, ImportFrom
from libcst.helpers import get_full_name_for_node_or_raise
from flay.common.exc import ParsingError


def file_to_node(file: Path | str) -> Module | None:
    if isinstance(file, str):
        file = Path(file)
    if file.match("*.so"):
        return None
    try:
        file_content = file.read_text()
        return parse_module(file_content)
    except UnicodeDecodeError as e:
        raise ParsingError(
            f"Could not open {file} because it's not encoded with {e.encoding}"
        )


def get_import_from_absolute_module_spec(
    node: ImportFrom, parent_package: str | None = None
) -> list[str]:
    module_node = node.module
    if module_node is None and not node.relative:
        raise ParsingError(
            f"No absolute module spec could be found for {node}, {parent_package}"
        )
    if module_node:
        return [
            resolve_name(
                "." * len(node.relative) + get_full_name_for_node_or_raise(module_node),
                parent_package,
            )
        ]
    elif parent_package is not None and len(node.relative) == 1:
        res = [parent_package]
        if not isinstance(node.names, ImportStar):
            for name in node.names:
                res.append(f"{parent_package}.{name.evaluated_name}")
        return res

    raise ParsingError(
        f"Don't know how to construct absolute module spec for node {node}"
    )
