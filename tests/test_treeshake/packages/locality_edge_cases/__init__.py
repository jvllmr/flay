from os import sep
from secrets import choice, token_urlsafe
from string import digits


def shadow_for():
    for sep in ["a"]:
        return sep
    return None


def shadow_with():
    with open(__file__) as choice:
        return choice


def shadow_except():
    try:
        return 1 / 1
    except ZeroDivisionError as token_urlsafe:
        return token_urlsafe
    return None


def shadow_match(value):
    match value:
        case {"key": sep}:
            return sep
    return None


digits = "123"


def use_choice() -> str:
    return choice(["a", digits])
