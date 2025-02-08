import typer
import typer as typerino

random_literal = "typer.Typer"

def modify_app(app: "typer.Typer") -> "typer.Typer":
    return app

def modify_app2(app2: "typerino.Typer") -> "typerino.Typer":
    return app2
