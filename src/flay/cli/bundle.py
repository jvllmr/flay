from __future__ import annotations
from pathlib import Path
from rich.progress import SpinnerColumn, TextColumn, MofNCompleteColumn
from flay.common.events import EventHandler
from flay.common.rich import console, check
from flay.bundle.package import (
    BundlePackageEvent,
    bundle_package,
    BundlePackageBundledMetadataEvent,
    BundlePackageFoundModuleEvent,
    BundlePackageFoundTotalModulesEvent,
    BundlePackageProcessModuleEvent,
)
import typing_extensions as te
import typing as t
from rich.progress import Progress, BarColumn


class BundlePackageCliIO(EventHandler[BundlePackageEvent]):
    found_modules_counter: int
    processed_modules: int
    total_modules: int | None
    progress: Progress

    def __init__(self, initial_module_spec: str) -> None:
        self.progress = Progress(
            SpinnerColumn(finished_text=check),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            MofNCompleteColumn(),
            TextColumn("[dim]({task.fields[spec]})"),
            console=console,
        )

        self.find_modules_task = self.progress.add_task(
            description="Searching for modules...", total=None, spec=initial_module_spec
        )
        self.process_modules_task = self.progress.add_task(
            visible=False, description="Processing modules...", spec=initial_module_spec
        )
        self.total_modules = None
        self.found_modules_counter = 0
        self.processed_modules = 0

    def on_event(self, event: BundlePackageEvent) -> None:
        if isinstance(event, BundlePackageFoundModuleEvent):
            self.found_modules_counter += 1
            self.progress.update(
                self.find_modules_task,
                spec=event.module_spec,
                completed=self.found_modules_counter,
            )
        elif isinstance(event, BundlePackageFoundTotalModulesEvent):
            self.total_modules = event.count
            self.progress.update(
                self.find_modules_task, completed=event.count, total=event.count
            )
        elif isinstance(event, BundlePackageProcessModuleEvent):
            self.processed_modules += 1
            if self.processed_modules <= (self.total_modules or 0):
                self.progress.update(
                    self.process_modules_task,
                    visible=True,
                    total=self.total_modules,
                    completed=self.processed_modules,
                    spec=event.module_spec,
                )
        elif isinstance(event, BundlePackageBundledMetadataEvent):
            self.end_progress()
            console.print(check, "Copied package metadata")

    def end_progress(self) -> None:
        if self.progress.live._started:
            self.progress.stop()

    def __enter__(self) -> te.Self:
        self.progress.start()
        return self

    def __exit__(self, *args: t.Any, **kw: t.Any) -> None:
        self.end_progress()


def cli_bundle_package(
    module_spec: str,
    output_path: Path,
    bundle_metadata: bool,
    resources: dict[str, str],
    import_aliases: dict[str, str],
) -> None:
    with BundlePackageCliIO(initial_module_spec=module_spec) as io:
        bundle_package(
            module_spec,
            output_path,
            bundle_metadata=bundle_metadata,
            resources=resources,
            import_aliases=import_aliases,
            event_handler=io,
        )


__all__ = ["cli_bundle_package"]
