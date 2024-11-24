from . import unused_file
from remove_unused_import import unused_file  # type: ignore  # noqa: F401, F811
import remove_unused_import.unused_file  # type: ignore  # noqa: F401
from .unused_file import unused_func
from remove_unused_import.unused_file import unused_func  # type: ignore  # noqa: F401, F811


def main() -> None:
    print("Hooray!")


if __name__ == "__main__":
    main()
