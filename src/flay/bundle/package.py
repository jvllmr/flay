from __future__ import annotations
from flay.common.compat import FLAY_STANDARD_ENCODING
from flay._flay_rs import FileCollector
from importlib.metadata import (
    Distribution,
    PackageNotFoundError,
    files as package_metadata_files,
)
from flay.common.compat import packages_distributions
from . import DEFAULT_BUNDLE_METADATA
from flay.common.module_spec import (
    find_all_files_in_module_spec,
    get_parent_package,
    get_top_level_package,
)
from pathlib import Path
import logging
import os.path
import typing as t
import shutil
from flay._flay_rs import transform_imports
import fnmatch
import sys

log = logging.getLogger(__name__)


def bundle_package(
    module_spec: str,
    destination_path: Path,
    bundle_metadata: bool = DEFAULT_BUNDLE_METADATA,
    resources: dict[str, str] | None = None,
    found_module_callback: t.Callable[[str], None] = lambda _: None,
    found_total_modules_callback: t.Callable[[int], None] = lambda _: None,
    process_module_callback: t.Callable[[str], None] = lambda _: None,
    bundled_metadata_callback: t.Callable[[], None] = lambda: None,
) -> None:
    resources = resources or {}
    collector = FileCollector(package=module_spec)

    for path in find_all_files_in_module_spec(module_spec):
        if path.match("*.py"):
            found_module_spec = (
                module_spec
                if path.name == "__init__.py"
                else f"{module_spec}.{path.stem}"
            )
            found_module_callback(found_module_spec)
            collector._process_module(found_module_spec)

    files = collector.collected_files
    found_total_modules_callback(len(files))
    top_level_package = get_top_level_package(module_spec)

    gitignore = destination_path / ".gitignore"
    if not gitignore.exists():
        gitignore.parent.mkdir(parents=True, exist_ok=True)
        gitignore.write_text("*")

    files_keys = set(files.keys())

    for found_module, found_path in files_keys:
        if found_path.match("*.py") and not found_path.match("*/__init__.py"):
            new_init_key = (
                get_parent_package(found_module),
                found_path.parent / "__init__.py",
            )
            if new_init_key not in files_keys and "__init__.py" in os.listdir(
                str(found_path.parent)
            ):
                # act as if an __init__.py exists
                files[new_init_key] = ""

    for (found_module, found_path), module_source in files.items():
        process_module_callback(found_module)
        if module_source:
            module_source = transform_imports(module_source)
        module_path_part = Path(os.path.sep.join(found_module.split(".")))

        if found_path.match(f"*/{module_path_part}/__init__.py"):
            target_file = destination_path / module_path_part / "__init__.py"
        else:
            target_file = destination_path / module_path_part.parent / found_path.name

        target_dir = target_file.parent
        if not target_dir.exists():
            target_dir.mkdir(parents=True)
        if module_source is not None:
            target_file.write_text(
                module_source,
                encoding=FLAY_STANDARD_ENCODING,
            )
            log.debug(
                "Written new source of %s to %s",
                found_path,
                target_file,
            )
        else:
            shutil.copy2(str(found_path), str(target_file))
            log.debug("Copied %s to %s", found_path, target_file)

        # look for {so_top_level_package}.libs dir and bundle it
        # i.e. the musllinux build of pydantic-core needs this
        if found_path.name.endswith(".so"):
            so_top_level_package = get_top_level_package(found_module)
            for sys_path in sys.path:
                if os.path.exists(sys_path) and os.path.isdir(sys_path):
                    for dir_ in os.listdir(sys_path):
                        if dir_ == f"{so_top_level_package}.libs":
                            shutil.copytree(
                                f"{sys_path}/{dir_}",
                                destination_path / dir_,
                                dirs_exist_ok=True,
                            )

    for module_spec, glob_pattern in resources.items():
        available_resources = package_metadata_files(module_spec)
        is_external = (
            get_top_level_package(module_spec=module_spec) != top_level_package
        )
        if available_resources:
            for resource in available_resources:
                resource_path = str(resource)

                if not fnmatch.fnmatch(resource_path, glob_pattern):
                    continue

                target_file = destination_path / resource_path

                target_dir = target_file.parent
                if not target_dir.exists():
                    target_dir.mkdir(parents=True)
                shutil.copy2(str(resource.locate()), str(target_file))
                log.debug("Copied %s to %s", found_path, target_file)

    if bundle_metadata:
        package_dists = packages_distributions()

        all_packages = {
            get_top_level_package(found_module) for (found_module, _) in files_keys
        }
        for package in all_packages:
            if package in package_dists:
                dist_names = package_dists[package]
            else:
                dist_names = [package]
            for dist_name in dist_names:
                try:
                    distribution = Distribution.from_name(dist_name)
                except PackageNotFoundError:  # pragma: no cover
                    log.warning("Could not locate dist-info for %s", dist_name)
                version = distribution.version
                dist_info_path = destination_path / f"{package}-{version}.dist-info"
                dist_info_path.mkdir(exist_ok=True)
                for metadata_file_name in ("METADATA", "PKG-INFO"):
                    if metadata := distribution.read_text(metadata_file_name):
                        metadata_path = dist_info_path / metadata_file_name
                        break
                else:  # pragma: no cover
                    raise PackageNotFoundError(module_spec)
                metadata_path.touch()
                metadata_path.write_text(metadata, encoding=FLAY_STANDARD_ENCODING)
        bundled_metadata_callback()
