from __future__ import annotations
from pathlib import Path
import typing as t
import os
import pytest
import sys
from flay.common.exc import FlayFileNotFoundError
from flay.bundle.package import bundle_package
import ast

if t.TYPE_CHECKING:
    from .conftest import RunBundlePackageT


def test_bundle_non_existing() -> None:
    with pytest.raises(FlayFileNotFoundError):
        bundle_package("non_existing_module", Path("sub_dir"))


def test_invalid_package(run_bundle_package: RunBundlePackageT) -> None:
    with pytest.raises(FlayFileNotFoundError):
        run_bundle_package("invalid_module", "invalid_module")


@pytest.mark.skipif(
    sys.platform.startswith("win"),
    reason="File content is not exactly the same on windows... Fix?",
)
def test_simple_bundle_hello_world(run_bundle_package: RunBundlePackageT) -> None:
    source_path, result_path = run_bundle_package("hello_world", "hello_world")

    for (sub_path1, dirs1, _files1), (sub_path2, dirs2, _files2) in zip(
        os.walk(str(source_path)), os.walk(str(result_path))
    ):
        assert dirs2 == dirs1
        files1 = sorted(_files1)
        files2 = sorted(_files2)
        assert files2 == files1
        for file1, file2 in zip(files1, files2):
            assert file2 == file1
            with open(sub_path1 + os.path.sep + file1) as f:
                content1 = f.read()

            with open(sub_path2 + os.path.sep + file2) as f:
                content2 = f.read()

            assert ast.dump(ast.parse(content2)) == ast.dump(ast.parse(content1))


def test_bundle_c_extension(run_bundle_package: RunBundlePackageT) -> None:
    source_path, result_path = run_bundle_package("fibunacci", "fibunacci")
    file_ending = ".pyd" if sys.platform.startswith("win") else ".so"
    lib_file: str | None = None
    for root_path in (source_path, result_path):
        for path, dirs, files in os.walk(str(root_path)):
            for file in files:
                if file.endswith(file_ending):
                    lib_file = file
                    break
        assert lib_file is not None
        assert lib_file.endswith(file_ending)
        assert lib_file.startswith("fibunacci_c")


@pytest.mark.parametrize(["vendor_module_name"], [("_vendor",), ("_bundled_packages",)])
def test_bundle_vendor_bundle(
    vendor_module_name: str, run_bundle_package: RunBundlePackageT
) -> None:
    _, result_path = run_bundle_package(
        "vendor_bundle", "vendor_bundle", vendor_module_name=vendor_module_name
    )

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert (
        f"import vendor_bundle.{vendor_module_name}.libcst as cst" in init_file_content
    )
    assert (
        f"from vendor_bundle.{vendor_module_name}.click import ClickException"
        in init_file_content
    )
    assert f"import vendor_bundle.{vendor_module_name}.typer" in init_file_content
    assert (
        f"from vendor_bundle.{vendor_module_name}.libcst.helpers import ensure_type"
        in init_file_content
    )
    assert f"import vendor_bundle.{vendor_module_name}.rich.emoji" in init_file_content

    assert (
        f"heart_emoji = vendor_bundle.{vendor_module_name}.rich.emoji.Emoji('heart')"
        in init_file_content
    )
    assert (
        f"vendor_bundle.{vendor_module_name}.typer.echo(heart_emoji)"
        in init_file_content
    )
    assert (
        f"vendor_bundle.{vendor_module_name}.typer.echo(heart_emoji)"
        in init_file_content
    )
    assert (
        "cst.parse_expression('assert answer_of_universe == 42')" in init_file_content
    )
    assert "tree = ensure_type(" in init_file_content
    assert "except ClickException:" in init_file_content
    assert (
        f"vendor_bundle.{vendor_module_name}.typer.echo('Something went wrong...')"
        in init_file_content
    )
    assert (
        f"from vendor_bundle.{vendor_module_name}.flay.cli.debug.bundle import debug_bundle_package"
        in init_file_content
    )
    assert "from pathlib import Path" in init_file_content


def test_bundle_transitive_init_file(run_bundle_package: RunBundlePackageT) -> None:
    _, result_path = run_bundle_package("transitive_init_file", "transitive_init_file")
    assert (result_path / "__init__.py").exists()
    assert (result_path / "module" / "abc.py").exists()
    assert (result_path / "module" / "def_.py").exists()
    assert (result_path / "module" / "__init__.py").exists()


def test_bundle_relative_imports(run_bundle_package: RunBundlePackageT) -> None:
    _, result_path = run_bundle_package("relative_imports", "relative_imports")
    assert (result_path / "module" / "hello_world.py").exists()
