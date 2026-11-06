from __future__ import annotations

import ast
import shutil
import sys
from pathlib import Path

from flay._flay_rs import clear_module_cache, fs_cache_stats, module_cache_len
from flay.bundle.package import bundle_package
from flay.treeshake.package import treeshake_package

PACKAGES_PATH = Path() / "tests" / "test_bundle" / "packages"


def _bundle(module_spec: str, pre_bundle_path: Path, destination_path: Path) -> None:
    sys.path = [str(pre_bundle_path), *sys.path]
    try:
        bundle_package(module_spec, destination_path, bundle_metadata=False)
    finally:
        sys.path = sys.path[1:]


def _setup_package(tmp_path: Path, package_name: str, module_spec: str) -> Path:
    pre_bundle_path = tmp_path / "pre_bundle"
    shutil.copytree(
        str(PACKAGES_PATH / package_name), str(pre_bundle_path / module_spec)
    )
    return pre_bundle_path


def test_module_cache_populated_and_cleared(tmp_path: Path) -> None:
    clear_module_cache()
    assert module_cache_len() == 0

    pre_bundle_path = _setup_package(tmp_path, "hello_world", "cache_populated")
    _bundle("cache_populated", pre_bundle_path, tmp_path / "bundled")

    assert module_cache_len() > 0
    clear_module_cache()
    assert module_cache_len() == 0


def test_bundle_with_warm_cache_matches_cold_cache(tmp_path: Path) -> None:
    pre_bundle_path = _setup_package(tmp_path, "hello_world", "cache_warm")

    clear_module_cache()
    _bundle("cache_warm", pre_bundle_path, tmp_path / "cold")
    cold_cache_len = module_cache_len()
    assert cold_cache_len > 0

    # second run reuses the cached modules, no new entries appear
    _bundle("cache_warm", pre_bundle_path, tmp_path / "warm")
    assert module_cache_len() == cold_cache_len

    cold_dir = tmp_path / "cold" / "cache_warm"
    warm_dir = tmp_path / "warm" / "cache_warm"
    for cold_file in sorted(cold_dir.rglob("*")):
        if not cold_file.is_file():
            continue
        warm_file = warm_dir / cold_file.relative_to(cold_dir)
        assert warm_file.exists(), cold_file
        if cold_file.suffix == ".py":
            assert ast.dump(ast.parse(warm_file.read_text())) == ast.dump(
                ast.parse(cold_file.read_text())
            )
        else:
            assert warm_file.read_bytes() == cold_file.read_bytes()


def test_module_cache_invalidated_on_file_change(tmp_path: Path) -> None:
    pre_bundle_path = _setup_package(tmp_path, "hello_world", "cache_invalidation")
    source_file = pre_bundle_path / "cache_invalidation" / "hello_world.py"

    clear_module_cache()
    _bundle("cache_invalidation", pre_bundle_path, tmp_path / "first")
    assert module_cache_len() > 0

    original_content = source_file.read_text()
    source_file.write_text(original_content + "\nchanged_marker = 1\n")

    _bundle("cache_invalidation", pre_bundle_path, tmp_path / "second")
    result_file = tmp_path / "second" / "cache_invalidation" / "hello_world.py"
    assert "changed_marker = 1" in result_file.read_text()


def _make_leaf_package(root: Path) -> Path:
    pkg = root / "leafpkg"
    pkg.mkdir(parents=True)
    (pkg / "__init__.py").write_text(
        "from .leaf import used_func\n"
        "\n"
        "\n"
        "def main() -> None:\n"
        "    print(used_func())\n"
        "\n"
        "\n"
        'if __name__ == "__main__":\n'
        "    main()\n"
    )
    (pkg / "leaf.py").write_text(
        "def used_func() -> int:\n    return 1\n\n\ndef unused_func() -> int:\n    return 2\n"
    )
    return pkg


def _snapshot(root: Path) -> dict[str, bytes]:
    return {
        str(p.relative_to(root)): p.read_bytes()
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }


def test_fs_cache_annotation_removal_hit_on_repeat_bundle(tmp_path: Path) -> None:
    pre_bundle_path = _setup_package(tmp_path, "type_annotations", "fs_ann")
    clear_module_cache()
    hits_before, _ = fs_cache_stats()
    _bundle("fs_ann", pre_bundle_path, tmp_path / "b1")
    _bundle("fs_ann", pre_bundle_path, tmp_path / "b2")
    hits_after, _ = fs_cache_stats()
    # the second bundle of unchanged sources is served from the fs cache
    assert hits_after > hits_before

    first_dir = tmp_path / "b1" / "fs_ann"
    second_dir = tmp_path / "b2" / "fs_ann"
    for first_file in sorted(first_dir.rglob("*.py")):
        second_file = second_dir / first_file.relative_to(first_dir)
        assert second_file.exists(), first_file
        assert ast.dump(ast.parse(first_file.read_text())) == ast.dump(
            ast.parse(second_file.read_text())
        )


def test_fs_cache_treeshake_repeat_run_hits(tmp_path: Path) -> None:
    pkg = _make_leaf_package(tmp_path)
    clear_module_cache()
    treeshake_package(str(tmp_path))
    first = _snapshot(pkg)
    leaf_content = (pkg / "leaf.py").read_text()
    assert "used_func" in leaf_content
    assert "unused_func" not in leaf_content

    # restore the original sources, drop the in-memory cache and re-run:
    # identical inputs must hit the persistent cache and give identical output
    shutil.rmtree(pkg)
    _make_leaf_package(tmp_path)
    clear_module_cache()
    hits_before, _ = fs_cache_stats()
    treeshake_package(str(tmp_path))
    hits_after, _ = fs_cache_stats()
    assert hits_after > hits_before
    assert _snapshot(pkg) == first


def test_fs_cache_leaf_body_change_only_busts_that_file(tmp_path: Path) -> None:
    pkg = _make_leaf_package(tmp_path)
    clear_module_cache()
    treeshake_package(str(tmp_path))

    shutil.rmtree(pkg)
    _make_leaf_package(tmp_path)
    # same symbols, only the body of the leaf changed
    (pkg / "leaf.py").write_text(
        "def used_func() -> int:\n    return 42\n\n\ndef unused_func() -> int:\n    return 2\n"
    )
    clear_module_cache()
    hits_before, misses_before = fs_cache_stats()
    treeshake_package(str(tmp_path))
    hits_after, misses_after = fs_cache_stats()

    leaf_content = (pkg / "leaf.py").read_text()
    assert "return 42" in leaf_content
    assert "unused_func" not in leaf_content
    # the aggregate counts entry and the changed leaf bust (2 misses), the
    # unchanged __init__.py is still served from the per-file cache (1 hit)
    assert misses_after - misses_before == 2
    assert hits_after - hits_before == 1


def test_fs_cache_leaf_symbol_change_busts_counts(tmp_path: Path) -> None:
    pkg = _make_leaf_package(tmp_path)
    clear_module_cache()
    treeshake_package(str(tmp_path))
    assert (pkg / "leaf.py").exists()

    shutil.rmtree(pkg)
    _make_leaf_package(tmp_path)
    # the leaf loses the symbol the package imports: counts must change
    (pkg / "leaf.py").write_text("def unused_func() -> int:\n    return 2\n")
    clear_module_cache()
    _, misses_before = fs_cache_stats()
    treeshake_package(str(tmp_path))
    _, misses_after = fs_cache_stats()
    # the aggregate counts cache entry must have been busted by the leaf change
    assert misses_after > misses_before
    # and the recomputed result is correct: the leaf had no referenced symbol
    # left and was removed entirely
    assert not (pkg / "leaf.py").exists()
    assert (pkg / "__init__.py").exists()
