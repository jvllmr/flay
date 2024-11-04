import typer

from flay.common.logging import setup_logger
from .debug import debug_app

app = typer.Typer(pretty_exceptions_enable=False)

app.add_typer(debug_app, name="debug")


@app.callback()
def app_main() -> None:
    setup_logger("")
