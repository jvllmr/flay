import contextlib
import os
import shutil
import sys
import typing as t
from pathlib import Path

import httpx2
import pytest
from pytest_benchmark.fixture import BenchmarkFixture

from flay.bundle.package import bundle_package
from flay.treeshake.package import treeshake_package

pytestmark = pytest.mark.skipif(
    not os.getenv("FLAY_INTEGRATION_TEST"),
    reason="Flay integration tests are not enabled. Enable them via setting FLAY_INTEGRATION_TEST env var.",
)


def _bundle_and_treeshake(
    package_name: str,
    target_path: Path,
    preserve_symbols: set[str] | None = None,
) -> None:
    shutil.rmtree(target_path, ignore_errors=True)
    target_path.mkdir(parents=True, exist_ok=True)
    bundle_package(package_name, target_path, bundle_metadata=True)
    treeshake_package(str(target_path), preserve_symbols=preserve_symbols)


@contextlib.contextmanager
def _bundled_sys_path(target_path: Path) -> t.Iterator[None]:
    old_sys_path = sys.path

    # no site-packages in sys_path to avoid collisions with system or virtual env packages
    new_sys_path = [path for path in sys.path if "site-packages" not in path] + [
        str(target_path.absolute())
    ]
    sys.path = new_sys_path
    try:
        yield
    finally:
        sys.path = old_sys_path


def test_integration_pre_commit(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    """
    Benchmark bundling/treeshaking pre-commit, then run it on this repository
    """
    target_path = tmp_path / "bundle"
    benchmark(_bundle_and_treeshake, "pre_commit", target_path)

    # the last benchmark round left a fresh bundle at target_path
    with _bundled_sys_path(target_path):
        from pre_commit.main import main  # type: ignore[import-untyped]

        main(["validate-config"])


@pytest.mark.skipif(
    sys.version_info < (3, 11),
    reason="fastapi requires python 3.11 or higher",
)
@pytest.mark.anyio
async def test_integration_fastapi(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    """
    Benchmark bundling/treeshaking fastapi, then define a route with a pydantic
    model and generate the openapi schema from the resulting bundle
    """
    preserve_symbols = {"fastapi.FastAPI"}
    target_path = tmp_path / "bundle"
    benchmark(
        _bundle_and_treeshake, "fastapi", target_path, preserve_symbols=preserve_symbols
    )

    # the last benchmark round left a fresh bundle at target_path
    with _bundled_sys_path(target_path):
        from fastapi import FastAPI
        from pydantic import BaseModel

        class Item(BaseModel):
            name: str
            price: float

        app = FastAPI()

        @app.get("/items/{item_id}")
        def get_item(item_id: int) -> Item:
            return Item(name="test", price=1.5)

        schema = app.openapi()
        item_schema = schema["components"]["schemas"]["Item"]
        assert item_schema["properties"] == {
            "name": {"title": "Name", "type": "string"},
            "price": {"title": "Price", "type": "number"},
        }
        assert (
            schema["paths"]["/items/{item_id}"]["get"]["operationId"]
            == "get_item_items__item_id__get"
        )

        async with httpx2.AsyncClient(
            transport=httpx2.ASGITransport(app=app)
        ) as client:
            response = await client.get("http://testserver/items/5")
        assert response.status_code == 200
        assert response.json() == {"name": "test", "price": 1.5}
