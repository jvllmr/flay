import typer
import typer as typerino
import collections.abc
random_literal = "typer.Typer"

def modify_app(app: "typer.Typer") -> "typer.Typer":
    return app

def modify_app2(app2: "typerino.Typer") -> "typerino.Typer":
    return app2

def accept_ordered_dict(ordered_dict: "collections.OrderedDict[str, str]") -> "collections.OrderedDict[str, str]":
    return ordered_dict

def accept_hashable(collection: "collections.abc.Hashable") -> "collections.abc.Hashable":
    return collection
