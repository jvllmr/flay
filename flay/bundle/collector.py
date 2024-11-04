from __future__ import annotations
from libcst import (
    Import,
    BatchableCSTVisitor,
    ImportFrom,
    visit_batched,
    Module,
)
from pathlib import Path
import typing as t

from flay.common.libcst import file_to_node, get_import_from_absolute_module_spec
from ..common.module_spec import find_module_path, get_parent_package
from stdlib_list import in_stdlib
import logging

log = logging.getLogger(__name__)


ModuleCollection = dict[tuple[str, Path], t.Optional[Module]]


class FileCollector(BatchableCSTVisitor):
    def __init__(
        self,
        package: str | None = None,
        already_collected: ModuleCollection | None = None,
    ) -> None:
        self._package = package
        self.collected_files: ModuleCollection = already_collected or {}

        super().__init__()

    def _process_module(self, module_spec: str) -> None:
        if in_stdlib(module_spec):
            return
        file_path = find_module_path(module_spec)

        if file_path is None or file_path.origin is None:
            log.warning(f"Don't know how to import {module_spec}. Skipping...")
            return
        file_path_origin = Path(file_path.origin)
        if (file_path.name, file_path_origin) in self.collected_files:
            return

        parsed_module_content = file_to_node(file_path.origin)
        self.collected_files[(file_path.name, file_path_origin)] = parsed_module_content
        log.debug(
            f"Collected {file_path.origin} from package {self._package} as {file_path.name}"
        )
        if parsed_module_content is None:
            return

        sub_collector = FileCollector(
            package=get_parent_package(file_path.name)
            if not file_path.origin.endswith("__init__.py")
            else file_path.name,
            already_collected=self.collected_files,
        )
        visit_batched(parsed_module_content, [sub_collector])
        self.collected_files.update(sub_collector.collected_files)

    def visit_Import(self, node: Import) -> None:
        for name in node.names:
            self._process_module(name.evaluated_name)

    def visit_ImportFrom(self, node: ImportFrom) -> None:
        for absolute_module in get_import_from_absolute_module_spec(
            node, self._package
        ):
            self._process_module(absolute_module)
