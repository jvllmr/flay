from __future__ import annotations
from libcst import visit_batched
from stdlib_list import in_stdlib
from flay.bundle.collector import FileCollector
from flay.common.libcst import file_to_node
from flay.common.logging import log_cst_code
from flay.common.module_spec import find_all_files_in_module_spec, get_top_level_package
from libcst import CSTTransformer, Import, ImportFrom
from libcst import Attribute, Name
from pathlib import Path
import typing as t
from libcst.helpers import get_absolute_module_for_import_or_raise
import logging
import os.path
from libcst.metadata import ScopeProvider
from libcst import MetadataWrapper
import shutil

log = logging.getLogger(__name__)


class ImportsTransformer(CSTTransformer):
    METADATA_DEPENDENCIES = (ScopeProvider,)

    def __init__(
        self, top_level_package: str, vendor_module_name: str = "_vendor"
    ) -> None:
        self.top_level_package = top_level_package
        self.vendor_module_name = vendor_module_name
        self._affected_attributes: list[Attribute] = []
        self._affected_names: list[Name] = []
        super().__init__()

    def _prepend_vendor(self, node: Attribute | Name) -> Attribute:
        if isinstance(node, Name):
            new_name = Attribute(
                value=Attribute(
                    value=Name(self.top_level_package),
                    attr=Name(self.vendor_module_name),
                ),
                attr=node,
            )
        else:
            deepest_attribute = node
            while not isinstance(deepest_attribute.value, Name):
                deepest_attribute = t.cast(Attribute, deepest_attribute.value)

            new_name = node.with_deep_changes(
                deepest_attribute,
                value=Attribute(
                    value=Attribute(
                        value=Name(self.top_level_package),
                        attr=Name(self.vendor_module_name),
                    ),
                    attr=deepest_attribute.value,
                ),
            )

        return new_name

    def _prepend_vendor_for_import(
        self,
        node: Attribute | Name,
        module_spec: str,
        references_need_update: bool = False,
    ) -> Attribute | Name:
        if module_spec.startswith(self.top_level_package) or in_stdlib(module_spec):
            return node
        if references_need_update:
            if isinstance(node, Name):
                self._affected_names.append(node)
            else:
                self._affected_attributes.append(node)
        return self._prepend_vendor(node)

    def leave_Import(self, original_node: Import, updated_node: Import) -> Import:
        new_node = updated_node.with_changes(
            names=[
                name.with_changes(
                    name=self._prepend_vendor_for_import(
                        name.name,
                        name.evaluated_name,
                        references_need_update=name.asname is None,
                    )
                )
                for name in updated_node.names
            ]
        )
        log.debug(
            "Transformed '%s' to '%s'",
            log_cst_code(original_node),
            log_cst_code(new_node),
        )
        return new_node

    def leave_ImportFrom(
        self, original_node: ImportFrom, updated_node: ImportFrom
    ) -> ImportFrom:
        if updated_node.module and not updated_node.relative:
            module_spec = get_absolute_module_for_import_or_raise(None, updated_node)
            new_node = updated_node.with_changes(
                module=self._prepend_vendor_for_import(updated_node.module, module_spec)
            )

            log.debug(
                "Transformed '%s' to '%s'",
                log_cst_code(original_node),
                log_cst_code(new_node),
            )
            return new_node
        return updated_node

    def leave_Name(self, original_node: Name, updated_node: Name) -> Name | Attribute:
        # TODO: we need to make that we are inside the scope of the original import
        for maybe_name in self._affected_names:
            if maybe_name.deep_equals(updated_node):
                new_node = self._prepend_vendor(updated_node)
                log.debug(
                    "Transformed Name '%s' to '%s'",
                    log_cst_code(updated_node),
                    log_cst_code(new_node),
                )
                return new_node
        return updated_node

    def leave_Attribute(
        self, original_node: Attribute, updated_node: Attribute
    ) -> Attribute:
        # TODO: we need to make that we are inside the scope of the original import
        for maybe_attribute in self._affected_attributes:
            if maybe_attribute.deep_equals(updated_node):
                new_node = self._prepend_vendor(updated_node)
                log.debug(
                    "Transformed Attribute '%s' to '%s'",
                    log_cst_code(updated_node),
                    log_cst_code(new_node),
                )
                return new_node
        return updated_node


def bundle_package(
    module_spec: str, destination_path: Path, vendor_module_name: str = "_vendor"
) -> None:
    collector = FileCollector(package=module_spec)
    for path in find_all_files_in_module_spec(module_spec):
        module = file_to_node(path)
        if module is not None:
            visit_batched(module, [collector])

    files = collector.collected_files
    top_level_package = get_top_level_package(module_spec)
    imports_transformer = ImportsTransformer(
        top_level_package=top_level_package,
        vendor_module_name=vendor_module_name,
    )
    for key, node in files.items():
        if node:
            files[key] = MetadataWrapper(node).visit(imports_transformer)

    vendor_path = destination_path / top_level_package / vendor_module_name

    gitignore = destination_path / ".gitignore"
    if not gitignore.exists():
        gitignore.parent.mkdir(parents=True, exist_ok=True)
        gitignore.write_text("*")

    for (found_module, found_path), module_node in files.items():
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
        if module_node:
            target_file.write_text(module_node.code)
        else:
            shutil.copy2(str(found_path), str(target_file))
