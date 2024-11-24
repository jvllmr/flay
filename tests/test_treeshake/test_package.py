from __future__ import annotations
from pathlib import Path
import typing as t
import os

TEST_DIR = Path(__file__).parent
TEST_PACKAGES_DIR = TEST_DIR / "packages"
if t.TYPE_CHECKING:
    from .conftest import RunTreeshakePackageT


def test_remove_unused_import(run_treeshake_package: RunTreeshakePackageT) -> None:
    result_path = run_treeshake_package(TEST_PACKAGES_DIR / "remove_unused_import")

    assert "unused_file.py" not in os.listdir(str(result_path))

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert "from . import unused_file" not in init_file_content
    assert "from remove_unused_import import unused_file" not in init_file_content
    assert "import remove_unused_import.unused_file" not in init_file_content
    assert "from .unused_file import unused_func" not in init_file_content
    assert (
        "from remove_unused_import.unused_file import unused_func"
        not in init_file_content
    )
