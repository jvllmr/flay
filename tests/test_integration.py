import pytest
import os
import sys
from flay.bundle.package import bundle_package
from flay.treeshake.package import treeshake_package
from pathlib import Path
import typing as t

pytestmark = pytest.mark.skipif(
    not os.getenv("FLAY_INTEGRATION_TEST"),
    reason="Flay integration tests are not enabled. Enable them via setting FLAY_INTEGRATION_TEST env var.",
)


@pytest.fixture(autouse=True)
def bundle_and_treeshake(
    request: pytest.FixtureRequest,
) -> t.Generator[None, t.Any, None]:
    test_name: str = request.function.__name__
    package_name = test_name.split("_", 2)[-1]
    target_path = Path("integration_bundle")
    if not target_path.exists():
        target_path.mkdir()
        (target_path / ".gitignore").write_bytes(b"*")

    bundle_package(package_name, target_path, bundle_metadata=True)
    treeshake_package(str(target_path))

    old_sys_path = sys.path

    # no site-packages in sys_path to avoid collisions with system or virtual env packages
    new_sys_path = [path for path in sys.path if "site-packages" not in path] + [
        str(target_path.absolute())
    ]
    sys.path = new_sys_path
    yield
    sys.path = old_sys_path


def test_integration_pre_commit() -> None:
    """
    Bundle/treeshake pre-commit and run it on this repository
    """

    from pre_commit.main import main  # type: ignore[import-untyped]

    main(["validate-config"])
