from flay.common.logging import (
    reset_logging_level,
    setup_logger,
    get_flay_logger,
    enable_debug_logging,
)
import logging
import pytest

local_log = logging.getLogger(__name__)
flay_log = get_flay_logger()


def test_logger_setup(caplog: pytest.LogCaptureFixture) -> None:
    with setup_logger("test_command"):
        local_log.debug("Test message")
        flay_log.debug("Flay message")
        enable_debug_logging()
        local_log.debug("Test message")
        flay_log.debug("Flay message")
        reset_logging_level()
        local_log.debug("Test message")
        flay_log.debug("Flay message")
    assert len(caplog.records) == 1
    assert caplog.records[0].levelno == logging.DEBUG
    assert caplog.messages[0] == "Flay message"
