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
    assert "from secrets import token_urlsafe, choice" not in init_file_content
    assert "from secrets import token_urlsafe" in init_file_content
    assert "import random, asyncio" not in init_file_content
    assert "import random" in init_file_content


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
    assert "def hello_world() -> None:\n    print('Hello world!')" in source_content

    unused_source_file = result_path / "unused_source.py"
    unused_source_content = unused_source_file.read_text()

    assert "def goodbye() -> None:" not in unused_source_content
    assert "def moin() -> None:\n    print('Moin Welt!')" in unused_source_content


def test_treeshake_package_remove_empty_modules(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "remove_empty_modules"
    result_path = run_treeshake_package(source_path)

    assert (result_path / "main.py").exists()
    assert (result_path / "__init__.py").exists()
    assert not (result_path / "unused").exists()


def test_treeshake_package_preserve_with_decorators(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "preserve_with_decorators"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()
    assert "@dataclass\nclass MyClass:\n    pass" in init_file_content
    assert "@contextmanager\ndef my_context_manager() ->" in init_file_content


def test_treeshake_package_re_exports(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "re_exports"
    result_path = run_treeshake_package(source_path)

    hello_world_init = result_path / "hello_world" / "__init__.py"
    assert hello_world_init.exists()
    hello_world_init_content = hello_world_init.read_text()
    assert "from .inner_hello_world import hello_world" in hello_world_init_content
    assert (
        "import re_exports.hello_world.inner_hello_world as inner_hello_world_alias"
        in hello_world_init_content
    )
    assert (
        "re_exports.hello_world.moin_world as moin_world_alias"
        not in hello_world_init_content
    )
    assert "useless_func" not in hello_world_init_content
    assert "from .moin_world import moin_world" not in hello_world_init_content

    assert not (result_path / "hello_world" / "moin_world").exists()

    inner_hello_world_init = hello_world_init = (
        result_path / "hello_world" / "inner_hello_world" / "__init__.py"
    )
    assert inner_hello_world_init.exists()
    inner_hello_world_init_content = inner_hello_world_init.read_text()
    assert (
        "def hello_world() -> None:\n    print('Hello World!')"
        in inner_hello_world_init_content
    )

    assert "def useless_func() -> None:" not in inner_hello_world_init_content


def test_treeshake_package_param_default_value(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "param_default_value"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()
    assert (
        "def Default(value: t.Any) -> DefaultPlaceholder:\n    return DefaultPlaceholder(value=value)"
        in init_file_content
    )
