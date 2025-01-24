from __future__ import annotations
from flay._flay_rs import FileCollector

from flay.common.module_spec import (
    find_all_files_in_module_spec,
    get_parent_package,
    get_top_level_package,
)
from pathlib import Path
import logging
import os.path

import shutil
import sys
from flay._flay_rs import transform_imports

log = logging.getLogger(__name__)


def bundle_package(
    module_spec: str, destination_path: Path, vendor_module_name: str = "_vendor"
) -> None:
    collector = FileCollector(package=module_spec)

    for path in find_all_files_in_module_spec(module_spec):
        if path.match("*.py"):
            found_module_spec = (
                module_spec
                if path.match("*/__init__.py")
                else f"{module_spec}.{path.stem}"
            )
            collector._process_module(found_module_spec)

    files = collector.collected_files
    top_level_package = get_top_level_package(module_spec)

    vendor_path = destination_path / top_level_package / vendor_module_name

    gitignore = destination_path / ".gitignore"
    if not gitignore.exists():
        gitignore.parent.mkdir(parents=True, exist_ok=True)
        gitignore.write_text("*")

    files_keys = set(files.keys())

    for found_module, _found_path in files_keys:
        found_path = Path(_found_path)
        if found_path.match("*.py") and not found_path.match("*/__init__.py"):
            new_init_key = (
                get_parent_package(found_module),
                str(found_path.parent / "__init__.py"),
            )
            if new_init_key not in files_keys and "__init__.py" in os.listdir(
                str(found_path.parent)
            ):
                files[new_init_key] = ""

    for (found_module, _found_path), module_source in files.items():
        found_path = Path(_found_path)

        if module_source:
            module_source = transform_imports(
                module_source, _found_path, top_level_package, vendor_module_name
            )
        module_path_part = Path(os.path.sep.join(found_module.split(".")))
        is_external = get_top_level_package(found_module) != top_level_package

        if found_path.match(f"*/{module_path_part}/__init__.py"):
            if is_external:
                target_file = vendor_path / module_path_part / "__init__.py"
            else:
                target_file = destination_path / module_path_part / "__init__.py"
        elif is_external:
            target_file = vendor_path / module_path_part.parent / found_path.name
        else:
            target_file = destination_path / module_path_part.parent / found_path.name

        target_dir = target_file.parent
        if not target_dir.exists():
            target_dir.mkdir(parents=True)
        if module_source is not None:
            target_file.write_text(
                module_source,
                encoding="utf-8" if sys.platform.startswith("win") else None,
            )
            log.debug(
                "Written new source of %s to %s",
                found_path,
                target_file,
            )
        else:
            shutil.copy2(str(found_path), str(target_file))
            log.debug("Copied %s to %s", found_path, target_file)
