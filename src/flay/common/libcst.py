from __future__ import annotations
from pathlib import Path
import libcst as cst
from flay.common.exc import ParsingError


def file_to_node(file: Path) -> cst.Module | None:
    if file.match("*.so") or file.match("*.pyd"):  # pragma: no cover
        return None
    try:
        file_content = file.read_bytes()
        return cst.parse_module(file_content)
    except UnicodeDecodeError as e:  # pragma: no cover
        raise ParsingError(
            f"Could not open {file} because it's not encoded with {e.encoding}"
        )
