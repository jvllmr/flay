from .class_builder import ClassBuilder
from .submodule import example
def main() -> None:
    builder = ClassBuilder()
    print(builder)
    example.example_func()

if __name__ == "__main__":
    main()
