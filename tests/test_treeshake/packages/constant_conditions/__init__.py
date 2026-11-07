import sys
import typing as t

if t.TYPE_CHECKING:
    from os import sep as type_checking_only

if sys.platform == "win32":
    PLATFORM = "windows"
else:
    PLATFORM = "linux"

if False:
    RESULT = "never"
elif True:
    RESULT = "always"
else:
    RESULT = "else"


def pick(flag: bool) -> str:
    if flag:
        return "yes"
    return "no"


def main() -> None:
    print(PLATFORM, RESULT, pick(True))


if __name__ == "__main__":
    main()
