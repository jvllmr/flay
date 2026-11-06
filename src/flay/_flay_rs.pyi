from pathlib import Path

def clear_module_cache() -> None: ...
def module_cache_len() -> int: ...
def fs_cache_stats() -> tuple[int, int]: ...

class FileCollector:
    package: str
    # maps (module spec, source path) to whether the file exists on disk;
    # False marks virtual __init__.py entries added on the python side
    collected_files: dict[tuple[str, Path], bool]

    def __init__(self, package: str, import_aliases: dict[str, str]) -> None: ...
    def _process_module(self, module_spec: str) -> None: ...

class ReferencesCounter:
    def __init__(
        self,
        preserve_symbols: set[str],
        import_aliases: dict[str, str],
        safe_decorators: set[str],
        cache_dir: Path | None = None,
    ): ...
    def visit_module(
        self,
        module_spec: str,
        source_path: Path,
    ) -> None: ...
    def reset_counter(self) -> None: ...
    def load_cached_counts(self, modules: list[tuple[str, Path]]) -> bool: ...
    def store_counts_cache(self, modules: list[tuple[str, Path]]) -> None: ...
    def into_nodes_remover(
        self, known_modules: set[str], cache_dir: Path | None = None
    ) -> NodesRemover: ...
    new_references_count: int

class NodesRemover:
    statements_removed: int
    def process_module(self, module_spec: str, source_path: str) -> None: ...

class AnnotationsRemover:
    def __init__(
        self, annotation_ecosystem_names: set[str], cache_dir: Path | None = None
    ) -> None: ...
    def remove_annotations_in_file(
        self, source_path: Path, target_path: Path
    ) -> None: ...
