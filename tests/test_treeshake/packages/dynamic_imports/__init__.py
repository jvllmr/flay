import importlib
from clonf import clonf_click


@clonf_click
def main() -> None:
    hello_world_module = importlib.import_module("dynamic_imports.hello")
    hello_world_module.hello_world()


if __name__ == "__main__":
    main()
