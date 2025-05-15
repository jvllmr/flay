from __future__ import annotations
import logging
import logging.handlers
import platformdirs
import contextvars
import typing as t
from contextlib import contextmanager
from rich.logging import RichHandler
import uuid
from pathlib import Path
import hashlib

from .rich import console

logfile_path_context: contextvars.ContextVar[str] = contextvars.ContextVar(
    "_flay_logfile"
)


FORMATTER = logging.Formatter(fmt="%(asctime)s %(name)s %(levelname)s: %(message)s")


def get_flay_logger() -> logging.Logger:
    return logging.getLogger("flay")


@contextmanager
def setup_logger(command: str) -> t.Generator[None, t.Any, None]:
    flay_logger = get_flay_logger()
    stream_handler = RichHandler(console=console, log_time_format="[%Y-%m-%d %H:%M:%S]")

    logging_root_dir = platformdirs.user_log_dir("flay", ensure_exists=True)
    unique_id = hashlib.sha256(uuid.uuid4().bytes).hexdigest()[:8]
    logging_file_path = str(Path(logging_root_dir) / f"flay-{command}-{unique_id}.log")
    logfile_path_context.set(logging_file_path)
    file_handler = logging.FileHandler(logging_file_path)
    file_handler.setFormatter(FORMATTER)

    handlers: tuple[logging.Handler, ...] = (stream_handler, file_handler)
    for handler in handlers:
        flay_logger.addHandler(handler)

    yield


def enable_debug_logging() -> None:
    flay_logger = get_flay_logger()
    flay_logger.setLevel(logging.DEBUG)


def reset_logging_level() -> None:
    flay_logger = get_flay_logger()
    flay_logger.setLevel(logging.NOTSET)
