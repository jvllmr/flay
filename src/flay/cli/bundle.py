from __future__ import annotations
from pathlib import Path
from rich.progress import SpinnerColumn, TextColumn, MofNCompleteColumn
from flay.common.rich import console, check
from flay.bundle.package import bundle_package
import typing_extensions as te
import typing as t
from rich.progress import Progress, BarColumn


class BundlePackageCliIO:
    found_modules_counter: int
    processed_modules: int
    total_modules: int | None

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

    def on_found_module(self, spec: str) -> None:
        self.found_modules_counter += 1
        self.progress.update(
            self.find_modules_task, spec=spec, completed=self.found_modules_counter
        )

    def on_found_total_modules(self, count: int) -> None:
        self.total_modules = count
        self.progress.update(self.find_modules_task, completed=count, total=count)

    def on_process_module(self, spec: str) -> None:
        self.processed_modules += 1
        if self.processed_modules <= (self.total_modules or 0):
            self.progress.update(
                self.process_modules_task,
                visible=True,
                total=self.total_modules,
                completed=self.processed_modules,
                spec=spec,
            )

    def end_progress(self) -> None:
        if self.progress.live._started:
            self.progress.stop()

    def on_bundled_metadata(self) -> None:
        self.end_progress()
        console.print(check, "Copied package metadata")

    def __enter__(self) -> te.Self:
        self.progress.start()
        return self

    def __exit__(self, *args: t.Any, **kw: t.Any) -> None:
        self.end_progress()


def cli_bundle_package(
    module_spec: str,
    output_path: Path,
    vendor_module_name: str,
    bundle_metadata: bool,
    resources: dict[str, str],
) -> None:
    with BundlePackageCliIO(initial_module_spec=module_spec) as io:
        bundle_package(
            module_spec,
            output_path,
            vendor_module_name=vendor_module_name,
            bundle_metadata=bundle_metadata,
            resources=resources,
            found_module_callback=io.on_found_module,
            found_total_modules_callback=io.on_found_total_modules,
            process_module_callback=io.on_process_module,
            bundled_metadata_callback=io.on_bundled_metadata,
        )


__all__ = ["cli_bundle_package"]
