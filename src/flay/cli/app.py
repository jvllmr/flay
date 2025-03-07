import click
from .debug import debug_app


@click.group()
def app() -> None:
    pass


app.name = "flay"
app.add_command(debug_app)
