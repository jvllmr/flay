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
OS_RELEASE_FILE = Path("/etc/os-release")
IS_ALPINE = OS_RELEASE_FILE.exists() and "alpine" in OS_RELEASE_FILE.read_text().lower()


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
    source_path, result_path = run_bundle_package(
        "hello_world", "hello_world", remove_type_annotations=False
    )

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
    # TODO: re-design test since it does not really test transformer recursion anymore
    _, result_path = run_bundle_package(
        "correct_transformer_recursion", "correct_transformer_recursion"
    )

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert (
        'typer.style("Hello World!", fg=typer.colors.BRIGHT_MAGENTA)'
        in init_file_content
    )


def test_bundle_annotation_string_literals(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "annotation_string_literals",
        "annotation_string_literals",
        remove_type_annotations=False,
    )

    init_file = result_path / "__init__.py"
    init_file_content = init_file.read_text()

    assert 'random_literal = "typer.Typer"' in init_file_content

    assert 'def modify_app(app: "typer.Typer") -> "typer.Typer":\n' in init_file_content

    assert (
        'def modify_app2(app2: "typerino.Typer") -> "typerino.Typer":\n'
        in init_file_content
    )

    assert 'def modify_app(app: "typer.Typer") -> "typer.Typer":\n' in init_file_content

    assert (
        'def accept_ordered_dict(ordered_dict: "collections.OrderedDict[str, str]") -> "collections.OrderedDict[str, str]":\n'
        in init_file_content
    )

    assert (
        'def accept_hashable(collection: "collections.abc.Hashable") -> "collections.abc.Hashable":\n'
        in init_file_content
    )


def test_bundle_remove_type_annotations(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package("type_annotations", "type_annotations")

    init_file_content = (result_path / "__init__.py").read_text()

    # function parameter and return annotations are removed
    assert 'def annotated_function(first, second="x", *args, key=False, **kwargs):' in (
        init_file_content
    )
    # a variable annotation keeps its assignment, a bare annotation is removed
    assert "annotated_var = [1]" in init_file_content
    assert "bare_annotation" not in init_file_content


def test_bundle_remove_type_annotations_preserves_ecosystems(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package("type_annotations", "type_annotations")

    init_file_content = (result_path / "__init__.py").read_text()

    # classes of ecosystems reading annotations at runtime keep them
    assert "name: str" in init_file_content
    assert "value: int = 5" in init_file_content
    assert "x: int" in init_file_content
    assert "y: int" in init_file_content
    assert "a: int" in init_file_content
    # a plain class loses its annotations
    assert "x = 3" in init_file_content
    assert "x: int = 3" not in init_file_content


def test_bundle_remove_type_annotations_off(
    run_bundle_package: RunBundlePackageT,
) -> None:
    source_path, result_path = run_bundle_package(
        "type_annotations", "type_annotations", remove_type_annotations=False
    )

    source_content = (source_path / "__init__.py").read_text()
    result_content = (result_path / "__init__.py").read_text()

    assert ast.dump(ast.parse(result_content)) == ast.dump(ast.parse(source_content))


def test_bundle_package_resources(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "bundle_resources",
        "bundle_resources",
        resources={"pre_commit": "pre_commit/resources/*"},
    )
    assert (
        result_path / "../pre_commit/resources/empty_template_main.go"
    ).exists() is True
    assert (
        result_path / "../pre_commit/resources/empty_template_go.mod"
    ).exists() is True


def test_bundle_package_dynamic_imports(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "dynamic_imports",
        "dynamic_imports",
    )

    assert (result_path / "sub_module/abc.py").exists()
    assert (result_path / "sub_module/aliased.py").exists()
    assert not (result_path / "sub_module/useless.py").exists()
    assert (result_path / "../clonf/integrations/click/__init__.py").exists()


def test_bundle_package_import_aliases(
    run_bundle_package: RunBundlePackageT,
) -> None:
    _, result_path = run_bundle_package(
        "import_aliases",
        "import_aliases",
        import_aliases={
            "import_aliases.sub.bundle1.func1": "import_aliases.sub.bundle2.func1"
        },
    )

    assert (result_path / "__init__.py").exists()
    assert (result_path / "sub/__init__.py").exists()
    assert (result_path / "sub/bundle1.py").exists()
    assert (result_path / "sub/bundle2.py").exists()
    assert not (result_path / "sub/useless.py").exists()


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


@pytest.mark.skipif(not IS_ALPINE, reason="libs only present on musllinux")
@pytest.mark.parametrize("name", ["pydantic_core"])
def test_bundle_package_so_libs(tmp_path: Path, name: str) -> None:
    bundle_package(name, tmp_path)
    assert (tmp_path / f"{name}.libs").exists(), os.listdir(tmp_path)


@pytest.mark.skipif(not IS_ALPINE, reason="libs only present on musllinux")
def test_bundle_package_so_libs_external(tmp_path: Path) -> None:
    bundle_package("flay", tmp_path)
    assert (tmp_path / "pydantic_core.libs").exists(), os.listdir(tmp_path)
