from .app import flay
import typing as t
from flay.common.logging import logfile_path_context
import logging
from flay.common.logging import setup_logger
import sys

__all__ = ("cli",)
log = logging.getLogger(__name__)


def cli() -> t.Any:  # pragma: no cover
    try:
        with setup_logger("_".join(sys.argv[1:])):
            return flay()
    except Exception as e:
        log.exception(
            "Unexpected error occurred. Detailed log can be found at %s",
            logfile_path_context.get(),
        )
        raise e


if __name__ == "__main__":
    cli()
