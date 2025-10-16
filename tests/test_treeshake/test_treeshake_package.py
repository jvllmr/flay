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
    assert 'def hello_world() -> None:\n    print("Hello world!")' in source_content

    unused_source_file = result_path / "unused_source.py"
    unused_source_content = unused_source_file.read_text()

    assert "def goodbye() -> None:" not in unused_source_content
    assert 'def moin() -> None:\n    print("Moin Welt!")' in unused_source_content


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
        'def hello_world() -> None:\n    print("Hello World!")'
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


def test_transitive_star_import(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "transitive_star_import"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()
    assert "Loader =" in init_file_content

    imported_init_file = result_path / "imported" / "__init__.py"
    imported_init_file_content = imported_init_file.read_text()
    assert "from .import2 import *" in imported_init_file_content

    import2_file = result_path / "imported" / "import2.py"
    import2_file_content = import2_file.read_text()
    assert "from .source import *" in import2_file_content

    source_file = result_path / "imported" / "source.py"
    source_file_content = source_file.read_text()
    assert "class SafeLoader:\n    pass" in source_file_content


def test_reference_in_comp(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "reference_in_comp"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()
    assert "_meta =" in init_file_content


def test_treeshake_package_global_assignment(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "global_assignment"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert "ConditionalOptional.__new__.__defaults__ = False," in init_file_content
    assert (
        "ConditionalOptional.check = _get_check_conditional(_check_optional)"
        in init_file_content
    )
    assert (
        "ConditionalOptional.apply_default = _apply_default_conditional_optional"
        in init_file_content
    )
    assert (
        "ConditionalOptional.remove_default = _remove_default_conditional_optional"
        in init_file_content
    )


def test_treeshake_package_local_var(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "local_var"
    result_path = run_treeshake_package(source_path)
    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert "_, git_version_b, _ =" in init_file_content


def test_treeshake_package_module_import_as_value(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "module_import_as_value"
    result_path = run_treeshake_package(source_path)
    class_builder_content = (result_path / "class_builder.py").read_text()
    assert "class ClassBuilder" in class_builder_content
    assert '_DEFAULT_ON_SETATTR = setters.pipe("Hallo Welt!")' in class_builder_content

    example_file_content = (result_path / "submodule/example.py").read_text()
    assert "def example_func() -> None:" in example_file_content

    setters_file_content = (result_path / "setters.py").read_text()
    assert "def pipe(value: str) -> str:" in setters_file_content

    example2_file_content = (result_path / "example2.py").read_text()
    assert "def example_func2() -> None:" in example2_file_content


def test_treeshake_package_pep562(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "pep562"
    result_path = run_treeshake_package(source_path)

    dynamic_file_content = (result_path / "dynamic.py").read_text()
    assert "def __dir__() -> list[str]:" in dynamic_file_content
    assert "def __getattr__(name: str) -> t.Any:" in dynamic_file_content

    main_file_content = (result_path / "main.py").read_text()
    assert "class BaseModel:" in main_file_content


def test_treeshake_package_global_for_loop(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "global_for_loop"
    result_path = run_treeshake_package(source_path)

    init_file_content = (result_path / "__init__.py").read_text()
    assert "GLOBAL_LIST: list[str] = []" in init_file_content
    assert 'for char in "hello world":' in init_file_content


def test_treeshake_package_class_bases(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "class_bases"
    result_path = run_treeshake_package(source_path)

    init_file_content = (result_path / "__init__.py").read_text()
    assert "from collections import ChainMap" in init_file_content
    assert "KT = TypeVar('KT')" in init_file_content
    assert "VT = TypeVar('VT')" in init_file_content


def test_treeshake_package_call_func_on_module_in_class(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "call_func_on_module_in_class"
    result_path = run_treeshake_package(source_path)

    hello_func_file_content = (result_path / "hello_func.py").read_text()
    assert "def the_hello_func(who: str) -> str:" in hello_func_file_content


def test_treeshake_package_dynamic_imports(
    run_treeshake_package: RunTreeshakePackageT,
) -> None:
    source_path = TEST_PACKAGES_DIR / "dynamic_imports"
    result_path = run_treeshake_package(source_path)

    hello_file_content = (result_path / "hello.py").read_text()
    assert "def hello_world() -> None:\n    print(" in hello_file_content

    assert not (result_path / "useless.py").exists()
