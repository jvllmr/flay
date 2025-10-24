from __future__ import annotations
import pytest
import typing as t
from pathlib import Path
import shutil
from flay.treeshake.package import treeshake_package


class RunTreeshakePackageT(t.Protocol):
    def __call__(
        self,
        path: Path,
        preserve_symbols: set[str] | None = None,
        import_aliases: dict[str, str] | None = None,
    ) -> Path: ...


@pytest.fixture
def run_treeshake_package(tmp_path: Path) -> RunTreeshakePackageT:
    def _run_treeshake_package(
        path: Path,
        preserve_symbols: set[str] | None = None,
        import_aliases: dict[str, str] | None = None,
    ) -> Path:
        assert path.is_dir(), "Must specifiy a directory!"
        target_path = tmp_path / path.name
        shutil.copytree(str(path), str(target_path))
        treeshake_package(
            str(tmp_path),
            preserve_symbols=preserve_symbols,
            import_aliases=import_aliases,
        )
        return target_path

    return _run_treeshake_package
