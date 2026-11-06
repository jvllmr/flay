from __future__ import annotations

import logging
import os
from pathlib import Path

import typing_extensions as te

from flay._flay_rs import ReferencesCounter
from flay.common.events import NOOP_EVENT_HANDLER, Event, EventHandler
from flay.common.exc import FlayTreeshakeError
from flay.ecosystem.import_aliases import get_default_import_aliases
from flay.ecosystem.preserve_symbols import (
    enrich_preserve_symbols_from_import_aliases,
    get_default_preserve_symbols,
)
from flay.ecosystem.safe_decorators import get_default_safe_decorators

log = logging.getLogger(__name__)


class TreeshakePackageFoundModuleEvent(Event):
    module_spec: str


class TreeshakePackageTotalModulesEvent(Event):
    count: int


class TreeshakePackageReferencesIterationEvent(Event):
    iteration: int


class TreeshakePackageNodesRemovalEvent(Event):
    module_spec: str


TreeshakePackageEvent: te.TypeAlias = (
    TreeshakePackageFoundModuleEvent
    | TreeshakePackageTotalModulesEvent
    | TreeshakePackageReferencesIterationEvent
    | TreeshakePackageNodesRemovalEvent
)


def _process_modules(
    references_counter: ReferencesCounter,
    file_modules: list[str],
    known_module_specs: dict[str, str],
) -> None:
    for file_path in file_modules:
        module_spec = known_module_specs[file_path]

        log.debug("Start processing references for module %s", module_spec)
        try:
            references_counter.visit_module(
                module_spec=module_spec, source_path=Path(file_path)
            )
        except BaseException as exc:  # pragma: no cover  # noqa: BLE001
            raise FlayTreeshakeError(
                f"An error occurred when counting references in {module_spec} located at {file_path}",
                exc,
            )


def treeshake_package(
    source_dir: str,
    import_aliases: dict[str, str] | None = None,
    preserve_symbols: set[str] | None = None,
    safe_decorators: set[str] | None = None,
    cache_dir: Path | None = None,
    event_handler: EventHandler[TreeshakePackageEvent] = NOOP_EVENT_HANDLER,
) -> int:
    source_files: set[str] = set()
    known_module_specs: dict[str, str] = {}
    for path, dirs, files in os.walk(source_dir):
        for file in files:
            if file.endswith((".py", ".pyi")):
                file_path = f"{path}{os.path.sep}{file}"
                source_files.add(file_path)
                module_spec = (
                    file_path[len(source_dir) :]
                    .strip(os.path.sep)
                    .split(".")[0]
                    .replace(os.path.sep, ".")
                )

                if module_spec.endswith((".__init__", ".__main__")):
                    known_module_specs[file_path] = module_spec.rsplit(".", 1)[0]
                else:
                    known_module_specs[file_path] = module_spec
                event_handler.on_event(
                    TreeshakePackageFoundModuleEvent(
                        module_spec=known_module_specs[file_path]
                    )
                )

    file_modules: list[str] = sorted(
        source_files, key=lambda x: 1 if x.endswith("__init__.py") else 0
    )
    event_handler.on_event(TreeshakePackageTotalModulesEvent(count=len(file_modules)))

    new_references_count = 1

    aliases = get_default_import_aliases()
    if import_aliases:
        aliases.update(import_aliases)

    preserve_symbols = get_default_preserve_symbols().union(preserve_symbols or [])
    enrich_preserve_symbols_from_import_aliases(preserve_symbols, aliases)

    s_decs = get_default_safe_decorators()
    if safe_decorators:
        s_decs = s_decs.union(safe_decorators)

    # references_counts stays in rust the whole time; only the small
    # new_references_count scalar is read back per iteration
    references_counter = ReferencesCounter(
        preserve_symbols,
        import_aliases=aliases,
        safe_decorators=s_decs,
        cache_dir=cache_dir,
    )
    modules = [
        (known_module_specs[file_path], Path(file_path)) for file_path in file_modules
    ]
    # the counted references are a function of every input file, so a change
    # in any single file (a leaf included) busts the persistent cache entry
    if not references_counter.load_cached_counts(modules):
        treeshake_iteration = 1
        # count references until no new references get added
        while new_references_count:
            log.debug(
                "Treeshake reference counter iteration %s",
                treeshake_iteration,
            )
            event_handler.on_event(
                TreeshakePackageReferencesIterationEvent(iteration=treeshake_iteration)
            )
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
        references_counter.store_counts_cache(modules)

    # remove nodes without references; the counted references are moved into
    # the nodes remover inside rust and never transferred to python
    nodes_remover = references_counter.into_nodes_remover(
        set(known_module_specs.values()), cache_dir
    )
    for file_path in file_modules:
        module_spec = known_module_specs[file_path]

        event_handler.on_event(
            TreeshakePackageNodesRemovalEvent(module_spec=module_spec)
        )
        nodes_remover.process_module(module_spec=module_spec, source_path=file_path)

    return nodes_remover.statements_removed
