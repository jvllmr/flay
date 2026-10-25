import pytest
import os
import sys
import httpx2
from flay.bundle.package import bundle_package
from flay.treeshake.package import treeshake_package
from pathlib import Path
import typing as t

pytestmark = pytest.mark.skipif(
    not os.getenv("FLAY_INTEGRATION_TEST"),
    reason="Flay integration tests are not enabled. Enable them via setting FLAY_INTEGRATION_TEST env var.",
)

# treeshake needs a live root inside the bundle; pre-commit has its
# `if __name__ == "__main__"` guard, fastapi needs an explicitly preserved
# entry symbol
_INTEGRATION_PRESERVE_SYMBOLS: dict[str, set[str]] = {
    "test_integration_fastapi": {"fastapi.FastAPI"},
}


@pytest.fixture(autouse=True)
def bundle_and_treeshake(
    request: pytest.FixtureRequest, tmp_path: Path
) -> t.Generator[None, t.Any, None]:
    test_name: str = request.function.__name__
    package_name = test_name.split("_", 2)[-1]
    target_path = tmp_path

    bundle_package(package_name, target_path, bundle_metadata=True)
    treeshake_package(
        str(target_path), preserve_symbols=_INTEGRATION_PRESERVE_SYMBOLS.get(test_name)
    )

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


@pytest.mark.skipif(
    sys.version_info < (3, 11),
    reason="fastapi requires python 3.11 or higher",
)
@pytest.mark.anyio
async def test_integration_fastapi() -> None:
    """
    Bundle/treeshake fastapi, define a route with a pydantic model and
    generate the openapi schema from it
    """

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

    async with httpx2.AsyncClient(transport=httpx2.ASGITransport(app=app)) as client:
        response = await client.get("http://testserver/items/5")
    assert response.status_code == 200
    assert response.json() == {"name": "test", "price": 1.5}
