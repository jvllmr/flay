from __future__ import annotations
from rich.console import Console
from rich.style import Style
from rich.text import Text


console = Console()


def ansi_style_text(text: str, style: Style | str = "") -> str:
    text_obj = Text(text=text, style=style)
    with console.capture() as capture:
        console.print(text_obj)
    return capture.get()


check = Text("✔", style=Style(color="green"))
