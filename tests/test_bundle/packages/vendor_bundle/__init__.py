import libcst as cst  # noqa: F401
from click import ClickException  # noqa: F401
import typer  # noqa: F401
from libcst.helpers import ensure_type  # noqa: F401
import rich.emoji  # noqa: F401

try:
    heart_emoji = rich.emoji.Emoji("heart")
    typer.echo(heart_emoji)
    tree = ensure_type(
        cst.parse_expression("assert answer_of_universe == 42"), cst.Assert
    )
except ClickException:
    typer.echo("Something went wrong...")
