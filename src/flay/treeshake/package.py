from __future__ import annotations

from flay._flay_rs import NodesRemover, ReferencesCounter


import os
from collections import defaultdict
import typing as t
import logging


log = logging.getLogger(__name__)


def treeshake_package(
    source_dir: str, preserve_packages: t.Collection[str] | None = None
) -> dict[str, int]:
    stats: dict[str, int] = defaultdict(int)
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

    file_modules: list[str] = sorted(
        source_files, key=lambda x: 1 if x.endswith("__init__.py") else 0
    )
    references_counts: dict[str, int] = defaultdict(int)
    new_references_count = 1

    references_counter = ReferencesCounter(references_counts)
    treeshake_iteration = 1
    # count references until no new references get added
    # NOTE: maybe follow imports instead? should be taken into consideration for future improvements
    while new_references_count:
        log.debug(
            "Treeshake reference counter iteration %s",
            treeshake_iteration,
        )
        treeshake_iteration += 1
        references_counter.reset_counter()
        for file_path in file_modules:
            module_spec = known_module_specs[file_path]

            log.debug("Start processing referencs for module %s", module_spec)
            references_counter.visit_module(
                module_spec=module_spec, source_path=file_path
            )

        new_references_count = references_counter.new_references_count
    references_counts |= references_counter.references_counts

    # remove nodes without references
    nodes_remover = NodesRemover(references_counts, set(known_module_specs.values()))
    for file_path in file_modules:
        module_spec = known_module_specs[file_path]

        nodes_remover.process_module(module_spec=module_spec, source_path=file_path)
        """
        if not new_module.body and not file_path.endswith("__init__.py"):
            os.remove(file_path)
            log.debug("Removed file %s", file_path)
            stats["Module"] += 1
            safe_remove_empty_dir(file_path)
        elif (
            not new_module.body
            and file_path.endswith("__init__.py")
            and len(os.listdir(os.path.dirname(file_path))) == 1
        ):
            os.remove(file_path)
            log.debug("Removed file %s", file_path)
            stats["Module"] += 1
            safe_remove_empty_dir(file_path)
        else:
            with open(file_path, "w") as f:
                f.write(new_module.code)
            log.debug("Processed code of %s", file_path)
        """
    return stats
