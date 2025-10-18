import pytest
import typing as t
from pathlib import Path
import shutil
from flay.treeshake.package import treeshake_package
from flay.bundle.package import bundle_package

import sys


class RunTreeshakePackageT(t.Protocol):
    def __call__(self, path: Path, bundle_before: bool = False) -> Path: ...


@pytest.fixture
def run_treeshake_package(tmp_path: Path) -> RunTreeshakePackageT:
    def _run_treeshake_package(path: Path, bundle_before: bool = False) -> Path:
        assert path.is_dir(), "Must specifiy a directory!"
        target_path = tmp_path / path.name
        if bundle_before:
            sys.path = [str(path.parent), *sys.path]
            try:
                bundle_package(path.name, target_path.parent)
            finally:
                sys.path[1:]
        else:
            shutil.copytree(str(path), str(target_path))
        treeshake_package(str(tmp_path))
        return target_path

    return _run_treeshake_package
