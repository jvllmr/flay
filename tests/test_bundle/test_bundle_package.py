from __future__ import annotations
from pathlib import Path
import typing as t
import os
import pytest
import sys
from flay.common.exc import FlayFileNotFoundError
from flay.bundle.package import bundle_package
import ast
from importlib.metadata import Distribution, requires
from packaging.requirements import Requirement

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
    _, result_path = run_bundle_package("fibunacci", "fibunacci")
    if sys.platform.startswith("win"):
        assert len(list(result_path.glob("fibunacci_c.*.pyd"))) == 1
    else:
        assert len(list(result_path.glob("fibunacci_c.*.so"))) == 1


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
        f'heart_emoji = vendor_bundle.{vendor_module_name}.rich.emoji.Emoji("heart")'
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
        'cst.parse_expression("assert answer_of_universe == 42")' in init_file_content
    )
    assert "tree = ensure_type(" in init_file_content
    assert "except ClickException:" in init_file_content
    assert (
        f'vendor_bundle.{vendor_module_name}.typer.echo("Something went wrong...")'
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


def test_bundle_correct_transformer_recursion(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "correct_transformer_recursion", "correct_transformer_recursion"
    )

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert (
        'correct_transformer_recursion._vendor.typer.style("Hello World!", fg=correct_transformer_recursion._vendor.typer.colors.BRIGHT_MAGENTA)'
        in init_file_content
    )


def test_bundle_annotation_string_literals(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "annotation_string_literals", "annotation_string_literals"
    )

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert 'random_literal = "typer.Typer"' in init_file_content

    assert (
        'def modify_app(app: "annotation_string_literals._vendor.typer.Typer") -> "annotation_string_literals._vendor.typer.Typer":\n'
        in init_file_content
    )

    assert (
        'def modify_app2(app2: "typerino.Typer") -> "typerino.Typer":\n'
        in init_file_content
    )


def test_bundle_package_resources(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "bundle_resources",
        "bundle_resources",
        resources={"pre_commit": "pre_commit/resources/*"},
    )
    assert (
        result_path / "_vendor/pre_commit/resources/empty_template_main.go"
    ).exists() is True
    assert (
        result_path / "_vendor/pre_commit/resources/empty_template_go.mod"
    ).exists() is True


def test_bundle_package_bundle_metadata(tmp_path: Path) -> None:
    bundle_package("flay", tmp_path, bundle_metadata=True)
    root_dist = Distribution.from_name("flay")
    assert (tmp_path / f"flay-{root_dist.version}.dist-info").exists()
    reqs = requires("flay")
    assert reqs is not None
    for pathname in os.listdir(tmp_path):
        assert not pathname.startswith("__future__")
    for req_ in reqs:
        req = Requirement(req_)
        if req.marker is not None and not req.marker.evaluate():
            continue
        dist = Distribution.from_name(req.name)
        assert hasattr(dist, "_path")
        assert (tmp_path / Path(dist._path).name).exists(), os.listdir(tmp_path)


@pytest.mark.skipif("alpine" not in Path("/etc/os-release").read_text().lower())
def test_bundle_package_so_libs(tmp_path: Path) -> None:
    bundle_package("pydantic-core", tmp_path)
    assert (tmp_path / "pydantic_core.libs").exists()


@pytest.mark.skipif("alpine" not in Path("/etc/os-release").read_text().lower())
def test_bundle_package_so_libs_external(tmp_path: Path) -> None:
    bundle_package("flay", tmp_path)
    assert (tmp_path / "flay/_vendor/pydantic_core.libs").exists()
