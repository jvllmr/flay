import sys


if sys.version_info < (3, 10):
    from importlib_metadata import packages_distributions  # type: ignore[import-not-found,unused-ignore]
else:
    from importlib.metadata import packages_distributions  # type: ignore[attr-defined]


__all__ = ["packages_distributions"]
