from pathlib import Path

import pytest


@pytest.fixture
def anyio_backend() -> str:
    return "asyncio"


@pytest.fixture(autouse=True)
def _isolated_fs_cache(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # keep the persistent fs cache of each test hermetic
    monkeypatch.setenv("FLAY_CACHE_DIR", str(tmp_path / "flay-cache"))
