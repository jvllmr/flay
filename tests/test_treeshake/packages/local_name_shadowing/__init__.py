from os import path
from secrets import choice, token_urlsafe


def shadowed_by_param(token_urlsafe: str) -> str:
    return token_urlsafe


def use_choice() -> str:
    return choice(["a", "b"])


def use_shadowed() -> str:
    path = "shadowed"
    return path
