from __future__ import annotations
from importlib.machinery import ModuleSpec
import sys
import typing as t
from pathlib import Path
from .exc import FlayFileNotFoundError
import os
from functools import cache


def get_top_level_package(module_spec: str) -> str:
    if "." not in module_spec:
        return module_spec
    return module_spec.split(".", 1)[0]


def get_parent_package(module_spec: str) -> str:
    if "." not in module_spec:
        return module_spec
    return ".".join(module_spec.split(".")[:-1])


VALID_FILE_EXTENSIONS = {
    "py",
    "so",  # shared objects
    "pyd",  # shared objects windows
}


def _path_can_lead_to_module_spec(
    path: str, file_name: str, module_spec_parts: list[str]
) -> bool:
    # only ancestors of the module directory can lead to the module; the
    # module directory itself would make the path finder look inside it
    for i in range(1, len(module_spec_parts) - 1):
        search_value = os.path.sep.join(module_spec_parts[: i + 1])
        if f"{path}{os.path.sep}{file_name}".endswith(search_value) or path.endswith(
            search_value
        ):
            return True
    return False


@cache
def _lookup_paths_for_module_spec(module_spec: str) -> list[str] | None:
    if "." not in module_spec:
        return None
    module_spec_parts = module_spec.split(".")
    top_level = module_spec_parts[0]
    paths: set[str] = set()
    for sys_path in sys.path:
        if os.path.isdir(sys_path) and top_level in os.listdir(sys_path):
            top_level_path = f"{sys_path}/{top_level}"
            if not os.path.isdir(top_level_path):
                continue
            # a directory without an __init__.py can be a namespace package
            paths.add(top_level_path)
            for path, dirs, files in os.walk(top_level_path):
                for file in files:
                    if "." not in file:
                        continue
                    file_name, file_extension = file.rsplit(".", 1)

                    if _path_can_lead_to_module_spec(
                        path, file_name, module_spec_parts
                    ) and (
                        file_extension in VALID_FILE_EXTENSIONS
                        or (
                            not path.endswith("__pycache__") and file_extension == "pyc"
                        )
                    ):
                        paths.add(path)

    return sorted(paths, key=lambda x: -len(x))


def _valid_module_spec(module_spec: str, result: ModuleSpec) -> bool:
    """
    rebuild the module spec from the found path and check if the searched value is equal
    """
    if result.origin is None:
        # namespace packages have no __init__.py and therefore no origin;
        # use one of their search locations instead
        if not result.submodule_search_locations:  # pragma: no cover
            return False
        origin = Path(next(iter(result.submodule_search_locations)))
    else:
        origin = Path(result.origin)
    segments_count = module_spec.count(".") + 1

    if origin.match("*/__init__.py"):
        origin = origin.parent
    built_module_spec_segments = []
    for _ in range(segments_count):
        built_module_spec_segments.append(origin.name.split(".", 1)[0])
        origin = origin.parent
    return ".".join(reversed(built_module_spec_segments)) == module_spec


def find_module_path(
    module_spec: str,
) -> ModuleSpec | None:
    assert "__init__" not in module_spec
    lookup_paths = _lookup_paths_for_module_spec(module_spec)

    for finder in sys.meta_path:
        try:
            result = finder.find_spec(module_spec, lookup_paths)
        except KeyError:
            # lookup of a module inside a not imported namespace package
            # raises a KeyError for the parent module
            continue

        if result is not None and _valid_module_spec(
            module_spec=module_spec, result=result
        ):
            return result

    return None


def find_all_files_in_module_spec(module_spec: str) -> t.Generator[Path, t.Any, None]:
    found_path = find_module_path(module_spec)
    if found_path is None:
        raise FlayFileNotFoundError(
            f"Could not find file for module spec '{module_spec}'. Found {found_path}."
        )
    if found_path.origin is None:
        # namespace packages have no __init__.py; collect the modules of all
        # their search locations
        for search_location in found_path.submodule_search_locations or []:
            for file in Path(search_location).iterdir():
                if file.match("*.py"):
                    yield file
        return
    module_init_file_path = Path(found_path.origin)

    module_folder_path = module_init_file_path.parent
    for file in module_folder_path.iterdir():
        if file.match("*.py"):
            yield module_folder_path / file
