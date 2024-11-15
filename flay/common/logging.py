from __future__ import annotations
import logging
import logging.handlers
from libcst import CSTNode, Module
import platformdirs
import tempfile
import contextvars
import typing as t
from contextlib import contextmanager
import typer
from queue import Queue

logfile_path_context: contextvars.ContextVar[str] = contextvars.ContextVar(
    "_flay_logfile"
)

LEVELNAME_FORMAT: dict[str, t.Callable[[str], str]] = {
    logging.getLevelName(logging.DEBUG): lambda level_name: typer.style(
        level_name, fg=typer.colors.BLUE
    ),
    logging.getLevelName(logging.INFO): lambda level_name: typer.style(
        level_name, fg=typer.colors.GREEN
    ),
    logging.getLevelName(logging.WARNING): lambda level_name: typer.style(
        level_name, fg=typer.colors.BRIGHT_YELLOW
    ),
    logging.getLevelName(logging.ERROR): lambda level_name: typer.style(
        level_name, fg=typer.colors.BRIGHT_RED
    ),
    logging.getLevelName(logging.CRITICAL): lambda level_name: typer.style(
        level_name, fg=typer.colors.RED, underline=True
    ),
}


class FlayFormatter(logging.Formatter):
    def __init__(self, colored: bool = False) -> None:
        self.colored = colored
        super().__init__(fmt="%(asctime)s %(name)s %(levelname)s: %(message)s")

    def formatTime(self, record: logging.LogRecord, datefmt: str | None = None) -> str:
        formatted = super().formatTime(record, datefmt)
        if self.colored:
            formatted = typer.style(formatted, fg=typer.colors.BRIGHT_MAGENTA)
        return formatted

    def format(self, record: logging.LogRecord) -> str:
        if self.colored:
            record.levelname = LEVELNAME_FORMAT[record.levelname](record.levelname)
        return super().format(record)


FORMATTER = FlayFormatter()
COLORED_FORMATTER = FlayFormatter(colored=True)


def get_flay_logger() -> logging.Logger:
    return logging.getLogger("flay")


class FlayQueueHandler(logging.handlers.QueueHandler):
    def prepare(self, record: logging.LogRecord) -> logging.LogRecord:
        # never pre-format unlike the default QueueHandler
        return record


@contextmanager
def setup_logger(command: str) -> t.Generator[None, t.Any, None]:
    queue: Queue[logging.LogRecord] = Queue()
    queue_handler = FlayQueueHandler(queue)

    flay_logger = get_flay_logger()
    stream_handler = logging.StreamHandler()
    stream_handler.setFormatter(COLORED_FORMATTER)

    logging_root_dir = platformdirs.user_log_dir("flay", ensure_exists=True)
    logging_file_path = tempfile.mktemp(".log", f"flay-{command}-", logging_root_dir)
    logfile_path_context.set(logging_file_path)
    file_handler = logging.FileHandler(logging_file_path)
    file_handler.setFormatter(FORMATTER)

    flay_logger.addHandler(queue_handler)
    handlers: tuple[logging.Handler, ...] = (stream_handler, file_handler)
    queue_listener = logging.handlers.QueueListener(
        queue, *handlers, respect_handler_level=False
    )
    queue_listener.start()
    yield
    queue_listener.stop()


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
