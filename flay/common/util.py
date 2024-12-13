import os

import logging

log = logging.getLogger(__name__)


def safe_remove_empty_dir(path: str) -> None:
    if os.path.isfile(path):
        path = os.path.dirname(path)
    while not os.path.exists(path):
        path = os.path.dirname(path)
    directory_path = path
    while directory_path:
        if not os.listdir(directory_path):
            os.rmdir(directory_path)
            log.debug(f"Removed directory {directory_path}")
            # TODO: never delete a root path just to be sure -> handling on windows?
            if directory_path != "/":
                directory_path = os.path.dirname(directory_path)
                continue
        break
