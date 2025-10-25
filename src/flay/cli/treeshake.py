from __future__ import annotations
from flay.treeshake.package import (
    TreeshakePackageEvent,
    treeshake_package,
    TreeshakePackageFoundModuleEvent,
    TreeshakePackageNodesRemovalEvent,
    TreeshakePackageReferencesIterationEvent,
    TreeshakePackageTotalModulesEvent,
)
from rich.progress import Progress
import typing as t
import typing_extensions as te
from flay.common.rich import console, check
from rich.progress import TextColumn, BarColumn, SpinnerColumn, MofNCompleteColumn
from flay.common.events import EventHandler


class TreeshakePackageCliIO(EventHandler[TreeshakePackageEvent]):
    def __init__(self) -> None:
        self.total_modules = 0
        self.found_modules = 0
        self.removal_processed_modules = 0
        self.references_iteration = 1
        self.progress = Progress(
            SpinnerColumn(finished_text=check),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            MofNCompleteColumn(),
            TextColumn("[dim]({task.fields[spec]})"),
            console=console,
        )
        self.discovery_task = self.progress.add_task(
            "Discovering modules...", total=None, spec=""
        )
        self.references_task = self.progress.add_task(
            "Counting references...", total=None, visible=False, spec=""
        )
        self.removal_task = self.progress.add_task(
            description="Removing unused source code",
            total=self.total_modules,
            spec="",
            visible=False,
        )

    def on_event(self, event: TreeshakePackageEvent) -> None:
        if isinstance(event, TreeshakePackageFoundModuleEvent):
            self.found_modules += 1
            self.progress.update(
                self.discovery_task,
                completed=self.found_modules,
                spec=event.module_spec,
            )
        elif isinstance(event, TreeshakePackageTotalModulesEvent):
            self.total_modules = event.count
            self.progress.update(
                self.discovery_task, completed=event.count, total=event.count
            )
        elif isinstance(event, TreeshakePackageReferencesIterationEvent):
            self.references_iteration = event.iteration
            self.progress.update(
                self.references_task,
                visible=True,
                completed=event.iteration,
            )
        elif isinstance(event, TreeshakePackageNodesRemovalEvent):
            self.progress.update(self.references_task, total=self.references_iteration)
            self.removal_processed_modules += 1
            self.progress.update(
                self.removal_task,
                total=self.total_modules,
                completed=self.removal_processed_modules,
                visible=True,
                spec=event.module_spec,
            )

    def end_progress(self) -> None:
        if self.progress.live._started:
            self.progress.stop()

    def __enter__(self) -> te.Self:
        self.progress.start()
        return self

    def __exit__(self, *args: t.Any, **kw: t.Any) -> None:
        self.end_progress()


def cli_treeshake_package(
    source_dir: str, import_aliases: dict[str, str], preserve_symbols: set[str]
) -> int:
    with TreeshakePackageCliIO() as io:
        return treeshake_package(
            source_dir=source_dir,
            import_aliases=import_aliases,
            preserve_symbols=preserve_symbols,
            event_handler=io,
        )


__all__ = ["cli_treeshake_package"]
