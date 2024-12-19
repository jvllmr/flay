from __future__ import annotations
from stdlib_list import in_stdlib
from flay._flay_rs import FileCollector
from flay.common.libcst import file_to_node
from flay.common.logging import log_cst_code
from flay.common.module_spec import find_all_files_in_module_spec, get_top_level_package
import libcst as cst
from pathlib import Path
import typing as t
from libcst.helpers import get_absolute_module_for_import_or_raise
import logging
import os.path

from libcst import MetadataWrapper
import shutil
import sys
from libcst.helpers import get_full_name_for_node

log = logging.getLogger(__name__)


class ImportsTransformer(cst.CSTTransformer):
    def __init__(
        self, top_level_package: str, vendor_module_name: str = "_vendor"
    ) -> None:
        self.top_level_package = top_level_package
        self.vendor_module_name = vendor_module_name
        self.reset()
        super().__init__()

    def reset(self) -> None:
        self._affected_names: set[str] = set()
        self.changes_count = 0

    def visit_Module(self, node: cst.Module) -> bool:
        self.reset()
        return True

    def _prepend_vendor(self, node: cst.Attribute | cst.Name) -> cst.Attribute:
        if isinstance(node, cst.Name):
            new_name = cst.Attribute(
                value=cst.Attribute(
                    value=cst.Name(self.top_level_package),
                    attr=cst.Name(self.vendor_module_name),
                ),
                attr=node,
            )
        else:
            deepest_attribute = node
            while not isinstance(deepest_attribute.value, cst.Name):
                deepest_attribute = t.cast(cst.Attribute, deepest_attribute.value)

            new_name = node.with_deep_changes(
                deepest_attribute,
                value=cst.Attribute(
                    value=cst.Attribute(
                        value=cst.Name(self.top_level_package),
                        attr=cst.Name(self.vendor_module_name),
                    ),
                    attr=deepest_attribute.value,
                ),
            )
        self.changes_count += 1
        return new_name

    def _prepend_vendor_for_import(
        self,
        node: cst.Attribute | cst.Name,
        module_spec: str,
        references_need_update: bool = False,
    ) -> cst.Attribute | cst.Name:
        if module_spec.startswith(self.top_level_package) or in_stdlib(
            get_top_level_package(module_spec)
        ):
            return node
        if references_need_update and (full_name := get_full_name_for_node(node)):
            self._affected_names.add(full_name)
        return self._prepend_vendor(node)

    def leave_Import(
        self, original_node: cst.Import, updated_node: cst.Import
    ) -> cst.Import:
        old_names = updated_node.names
        new_names = [
            name_val.with_changes(name=new_name_val)
            if (
                new_name_val := self._prepend_vendor_for_import(
                    name_val.name,
                    name_val.evaluated_name,
                    references_need_update=name_val.asname is None,
                )
            )
            is not name_val.name
            else name_val
            for name_val in updated_node.names
        ]
        for old_name, new_name in zip(old_names, new_names):
            if new_name is not old_name:
                new_node = updated_node.with_changes(names=new_names)
                log.debug(
                    "Transformed Import: '%s' => '%s'",
                    log_cst_code(original_node),
                    log_cst_code(new_node),
                )
                return new_node
        return updated_node

    def leave_ImportFrom(
        self, original_node: cst.ImportFrom, updated_node: cst.ImportFrom
    ) -> cst.ImportFrom:
        if updated_node.module and not updated_node.relative:
            module_spec = get_absolute_module_for_import_or_raise(None, updated_node)
            old_module = updated_node.module
            new_module = self._prepend_vendor_for_import(
                updated_node.module, module_spec
            )
            if new_module is not old_module:
                new_node = updated_node.with_changes(module=new_module)
                log.debug(
                    "Transformed ImportFrom: '%s' => '%s'",
                    log_cst_code(original_node),
                    log_cst_code(new_node),
                )
                return new_node
        return updated_node

    @t.overload
    def _prepend_vendor_to_name(
        self, original_node: cst.Name, updated_node: cst.Name
    ) -> cst.Name: ...

    @t.overload
    def _prepend_vendor_to_name(
        self, original_node: cst.Attribute, updated_node: cst.Attribute
    ) -> cst.Attribute: ...

    def _prepend_vendor_to_name(
        self,
        original_node: cst.Name | cst.Attribute,
        updated_node: cst.Name | cst.Attribute,
    ) -> cst.Attribute | cst.Name:
        # TODO: we need to make sure that we are inside the scope of the original import
        full_name = get_full_name_for_node(updated_node)
        if full_name in self._affected_names:
            new_node = self._prepend_vendor(updated_node)
            log.debug(
                "Transformed %s: '%s' => '%s'",
                original_node.__class__.__name__,
                log_cst_code(updated_node),
                log_cst_code(new_node),
            )
            return new_node

        return updated_node

    def leave_Name(
        self, original_node: cst.Name, updated_node: cst.Name
    ) -> cst.Name | cst.Attribute:
        return self._prepend_vendor_to_name(original_node, updated_node)

    def leave_Attribute(
        self, original_node: cst.Attribute, updated_node: cst.Attribute
    ) -> cst.Attribute:
        return self._prepend_vendor_to_name(original_node, updated_node)


def bundle_package(
    module_spec: str, destination_path: Path, vendor_module_name: str = "_vendor"
) -> None:
    collector = FileCollector(package=module_spec)

    for path in find_all_files_in_module_spec(module_spec):
        module = file_to_node(path)

        if module is not None:
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
    imports_transformer = ImportsTransformer(
        top_level_package=top_level_package,
        vendor_module_name=vendor_module_name,
    )
    for (found_module, _found_path), module_source in files.items():
        found_path = Path(_found_path)
        if module_source:
            module_node = file_to_node(path)
            assert module_node is not None
            module_node = MetadataWrapper(module_node, unsafe_skip_copy=True).visit(
                imports_transformer
            )
        else:
            module_node = None
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
        if imports_transformer.changes_count and module_node:
            target_file.write_text(
                module_node.code,
                encoding="utf-8" if sys.platform.startswith("win") else None,
            )
            log.debug(f"Written new CST of {found_path} to {target_file}")
        else:
            shutil.copy2(str(found_path), str(target_file))
            log.debug(f"Copied {found_path} to {target_file}")
