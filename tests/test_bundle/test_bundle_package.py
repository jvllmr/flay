from __future__ import annotations
import typing as t
import os
from .conftest import dos2unix

if t.TYPE_CHECKING:
    from .conftest import RunBundlePackageT


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
                content1 = dos2unix(f.read())

            with open(sub_path2 + "/" + file2) as f:
                content2 = dos2unix(f.read())

            assert content2 == content1
