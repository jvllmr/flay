import sys


if sys.version_info < (3, 11):
    # packages_distributions is available, but does not work as expected with cpython@3.10 (rich was not discovered)
    from importlib_metadata import packages_distributions  # type: ignore[import-not-found,unused-ignore]
else:
    from importlib.metadata import packages_distributions  # type: ignore[attr-defined]


__all__ = ["packages_distributions"]
