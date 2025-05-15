from flay.treeshake.package import treeshake_package
from rich.progress import Progress
import typing as t
import typing_extensions as te
from flay.common.rich import console, check
from rich.progress import TextColumn, BarColumn, SpinnerColumn, MofNCompleteColumn


class TreeshakePackageCliIO:
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

    def on_found_module(self, spec: str) -> None:
        self.found_modules += 1
        self.progress.update(
            self.discovery_task, completed=self.found_modules, spec=spec
        )

    def on_total_modules(self, count: int) -> None:
        self.total_modules = count
        self.progress.update(self.discovery_task, completed=count, total=count)

    def on_references_iteration(self, count: int) -> None:
        self.references_iteration = count
        self.progress.update(
            self.references_task,
            visible=True,
            completed=count,
        )

    def on_nodes_removal(self, spec: str) -> None:
        self.progress.update(self.references_task, total=self.references_iteration)
        self.removal_processed_modules += 1
        self.progress.update(
            self.removal_task,
            total=self.total_modules,
            completed=self.removal_processed_modules,
            visible=True,
            spec=spec,
        )

    def end_progress(self) -> None:
        if self.progress.live._started:
            self.progress.stop()

    def __enter__(self) -> te.Self:
        self.progress.start()
        return self

    def __exit__(self, *args: t.Any, **kw: t.Any) -> None:
        self.end_progress()


def cli_treeshake_package(source_dir: str) -> int:
    with TreeshakePackageCliIO() as io:
        return treeshake_package(
            source_dir=source_dir,
            found_module_callback=io.on_found_module,
            total_modules_callback=io.on_total_modules,
            nodes_removal_callback=io.on_nodes_removal,
            references_iteration_callback=io.on_references_iteration,
        )


__all__ = ["cli_treeshake_package"]
