# nopycln: file
from . import unused_file
from remove_unused_import import unused_file  # type: ignore  # noqa: F401, F811
import remove_unused_import.unused_file  # type: ignore  # noqa: F401
from .unused_file import unused_func
from remove_unused_import.unused_file import unused_func  # type: ignore  # noqa: F401, F811
from secrets import token_urlsafe, choice  # noqa: F401
import random, asyncio  # noqa: E401, F401


def main() -> None:
    print("Hooray!")
    print("Is this your token?", token_urlsafe())
    print("Is this your lucky number?", random.random())


if __name__ == "__main__":
    main()
