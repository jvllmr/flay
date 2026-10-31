import sys
from importlib.metadata import packages_distributions

FLAY_STANDARD_ENCODING = "utf-8" if sys.platform.startswith("win") else None


__all__ = ["packages_distributions"]
