from __future__ import annotations
from pathlib import Path
import typing as t
import os


TEST_DIR = Path(__file__).parent
TEST_PACKAGES_DIR = TEST_DIR / "packages"
if t.TYPE_CHECKING:
    from .conftest import RunTreeshakePackageT


def test_treeshake_package_remove_unused_import(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
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


def test_treeshake_package_import_star(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "import_star"
    result_path = run_treeshake_package(source_path)

    main_file = result_path / "__main__.py"
    main_content = main_file.read_text()

    assert "from .source import *" in main_content

    source_file = result_path / "source.py"
    source_content = source_file.read_text()

    assert "from .unused_source import *" in source_content
    assert 'def hello_world() -> None:\n    print("Hello world!")' in source_content

    unused_source_file = result_path / "unused_source.py"
    unused_source_content = unused_source_file.read_text()

    assert "def goodbye() -> None:" not in unused_source_content
    assert 'def moin() -> None:\n    print("Moin Welt!")' in unused_source_content
