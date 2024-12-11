from __future__ import annotations
from pathlib import Path
import typing as t
import os
import pytest
import sys
from flay.common.exc import FlayFileNotFoundError
from flay.bundle.package import bundle_package

if t.TYPE_CHECKING:
    from .conftest import RunBundlePackageT


def test_bundle_non_existing() -> None:
    with pytest.raises(FlayFileNotFoundError):
        bundle_package("non_existing_module", Path("sub_dir"))


def test_invalid_package(run_bundle_package: RunBundlePackageT) -> None:
    with pytest.raises(FlayFileNotFoundError):
        run_bundle_package("invalid_module", "invalid_module")


@pytest.mark.skipif(sys.platform.startswith("win"), reason="Not working on windows...")
def test_simple_bundle_hello_world(run_bundle_package: RunBundlePackageT) -> None:
    source_path, result_path = run_bundle_package("hello_world", "hello_world")

    for (sub_path1, dirs1, files1), (sub_path2, dirs2, files2) in zip(
        os.walk(str(source_path)), os.walk(str(result_path))
    ):
        assert dirs2 == dirs1
        assert files2 == files1
        for file1, file2 in zip(files1, files2):
            assert file2 == file1
            with open(sub_path1 + "/" + file1) as f:
                content1 = f.read()

            with open(sub_path2 + "/" + file2) as f:
                content2 = f.read()

            assert content2 == content1


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
