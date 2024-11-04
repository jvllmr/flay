from __future__ import annotations
import logging
from libcst import CSTNode, Module
import platformdirs
import tempfile
import contextvars
import typing as t

logfile_path_context: contextvars.ContextVar[str] = contextvars.ContextVar(
    "_flay_logfile"
)


def get_flay_logger() -> logging.Logger:
    return logging.getLogger("flay")


def setup_logger(command: str) -> None:
    flay_logger = get_flay_logger()
    stream_handler = logging.StreamHandler()
    logging_root_dir = platformdirs.user_log_dir("flay", ensure_exists=True)
    logging_file_path = tempfile.mktemp(".log", f"flay-{command}", logging_root_dir)
    logfile_path_context.set(logging_file_path)
    file_handler = logging.FileHandler(logging_file_path)
    flay_logger.addHandler(stream_handler)
    flay_logger.addHandler(file_handler)


def enable_debug_logging() -> None:
    flay_logger = get_flay_logger()
    flay_logger.setLevel(logging.DEBUG)


class _Serializable(t.Protocol):
    def __str__(self) -> str: ...


class LazyStr:
    def __init__(self, factory: t.Callable[[], str | _Serializable]):
        self.factory = factory

    def get_string(self) -> str:
        resolved = self.factory()
        if isinstance(resolved, str):
            return resolved
        return str(resolved)

    def __repr__(self) -> str:
        return f"<LazyStr resolved_value='{self.get_string()}' >"

    def __str__(self) -> str:
        return self.get_string()


def log_cst_code(node: CSTNode) -> LazyStr:
    """
    A helper method for when the code of a CST node needs to be logged.
    It returns a LazyStr which only evaluates the needed code when serialized.
    """

    def gen_str() -> str:
        fake_module = Module([node])  # type: ignore
        return fake_module.code_for_node(node)

    return LazyStr(gen_str)
