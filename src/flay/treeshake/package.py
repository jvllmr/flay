from __future__ import annotations

from flay._flay_rs import NodesRemover, ReferencesCounter
from pathlib import Path

import os
from collections import defaultdict
import typing as t
import logging
from .fixed import get_treeshake_fixed_preservations

log = logging.getLogger(__name__)


def _process_modules(
    references_counter: ReferencesCounter,
    file_modules: list[str],
    known_module_specs: dict[str, str],
) -> None:
    for file_path in file_modules:
        module_spec = known_module_specs[file_path]

        log.debug("Start processing references for module %s", module_spec)
        references_counter.visit_module(
            module_spec=module_spec, source_path=Path(file_path)
        )


def treeshake_package(
    source_dir: str,
    found_module_callback: t.Callable[[str], None] = lambda _: None,
    total_modules_callback: t.Callable[[int], None] = lambda _: None,
    references_iteration_callback: t.Callable[[int], None] = lambda _: None,
    nodes_removal_callback: t.Callable[[str], None] = lambda _: None,
) -> int:
    source_files: set[str] = set()
    known_module_specs: dict[str, str] = {}
    for path, dirs, files in os.walk(source_dir):
        for file in files:
            if file.endswith(".py") or file.endswith(".pyi"):
                file_path = f"{path}{os.path.sep}{file}"
                source_files.add(file_path)
                module_spec = (
                    file_path[len(source_dir) :]
                    .strip(os.path.sep)
                    .split(".")[0]
                    .replace(os.path.sep, ".")
                )

                if module_spec.endswith(".__init__") or module_spec.endswith(
                    ".__main__"
                ):
                    known_module_specs[file_path] = module_spec.rsplit(".", 1)[0]
                else:
                    known_module_specs[file_path] = module_spec
                found_module_callback(known_module_specs[file_path])

    file_modules: list[str] = sorted(
        source_files, key=lambda x: 1 if x.endswith("__init__.py") else 0
    )
    total_modules_callback(len(file_modules))
    references_counts: dict[str, int] = defaultdict(int)
    treeshake_fixed_preservations = get_treeshake_fixed_preservations()
    for preserved_name in treeshake_fixed_preservations:
        references_counts[preserved_name] = 1
    new_references_count = 1 + len(treeshake_fixed_preservations)

    references_counter = ReferencesCounter(references_counts)
    treeshake_iteration = 1
    # count references until no new references get added
    while new_references_count:
        log.debug(
            "Treeshake reference counter iteration %s",
            treeshake_iteration,
        )
        references_iteration_callback(treeshake_iteration)
        treeshake_iteration += 1
        references_counter.reset_counter()
        _process_modules(
            references_counter=references_counter,
            file_modules=file_modules,
            known_module_specs=known_module_specs,
        )
        new_references_count = references_counter.new_references_count
    _process_modules(
        references_counter=references_counter,
        file_modules=file_modules,
        known_module_specs=known_module_specs,
    )
    references_counts |= references_counter.references_counts

    log.debug("Counted references: %s", references_counts)

    # remove nodes without references
    nodes_remover = NodesRemover(references_counts, set(known_module_specs.values()))
    for file_path in file_modules:
        module_spec = known_module_specs[file_path]

        nodes_removal_callback(module_spec)
        nodes_remover.process_module(module_spec=module_spec, source_path=file_path)

    return nodes_remover.statements_removed
