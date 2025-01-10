import pytest
from pathlib import Path
import typing as t
from flay.bundle.package import bundle_package
import sys
import shutil
import os
import subprocess


class RunBundlePackageT(t.Protocol):
    def __call__(
        self, package_name: str, module_spec: str, vendor_module_name: str = "_vendor"
    ) -> tuple[Path, Path]: ...


PACKAGES_PATH = Path() / "tests" / "test_bundle" / "packages"


@pytest.fixture
def run_bundle_package(tmp_path: Path) -> RunBundlePackageT:
    def _run_bundle_package(
        package_name: str, module_spec: str, vendor_module_name: str = "_vendor"
    ) -> tuple[Path, Path]:
        pre_bundle_path = tmp_path / "pre_bundle"
        bundled_path = tmp_path / "bundled"
        shutil.copytree(
            str(PACKAGES_PATH / package_name), str(pre_bundle_path / package_name)
        )
        build_before = "pyproject.toml" in os.listdir(
            str(pre_bundle_path / package_name)
        )
        if build_before:
            install_cmd = [
                "pdm",
                "run",
                "pip",
                "install",
                str(pre_bundle_path / package_name),
            ]
            print(  # noqa: T201
                f"Running to install {package_name}:",
                " ".join(install_cmd),
            )
            subprocess.run(install_cmd)

        else:
            sys.path = [str(pre_bundle_path), *sys.path]
        try:
            bundle_package(
                module_spec=module_spec,
                destination_path=bundled_path,
                vendor_module_name=vendor_module_name,
            )
        finally:
            if build_before:
                subprocess.run(["pdm", "run", "pip", "uninstall", "-y", package_name])
            else:
                sys.path = sys.path[1:]
        return pre_bundle_path / package_name, bundled_path / package_name

    return _run_bundle_package
