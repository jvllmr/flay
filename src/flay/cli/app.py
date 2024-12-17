import typer

from .debug import debug_app

app = typer.Typer(pretty_exceptions_enable=False)

app.add_typer(debug_app, name="debug")
