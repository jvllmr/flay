import os
import nox

os.environ.update({"PDM_IGNORE_SAVED_PYTHON": "1"})


@nox.session(python=["3.10", "3.11", "3.12", "3.13"])
def tests(session: nox.Session) -> None:
    session.run_always("pdm", "install", "-G", "test", external=True)
    session.run("pytest")
