class FileCollector:
    package: str
    collected_files: dict[tuple[str, str], str | None]

    def __init__(self, package: str) -> None: ...
    def _process_module(self, module_spec: str) -> None: ...

def transform_imports(
    source_code: str, source_path: str, top_level_package: str, vendor_module_name: str
) -> str: ...

class ReferencesCounter:
    def __init__(self, references_counts: dict[str, int]): ...
    def visit_module(
        self,
        module_spec: str,
        source_path: str,
    ) -> None: ...
    def reset_counter(self) -> None: ...
    references_counts: dict[str, int]
    new_references_count: int
