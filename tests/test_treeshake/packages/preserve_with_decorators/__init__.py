from dataclasses import dataclass
from contextlib import contextmanager
import typing as t


def unknown_decorator(func: t.Callable[[], None]) -> None:
    print(func)

@unknown_decorator
def decorated_func() -> None:
    pass

@dataclass
class MyClass:
    @classmethod
    def classmeth(cls) -> None:
        pass

    @staticmethod
    def static() -> None:
        pass



@contextmanager
def my_context_manager() -> t.Generator[None,t.Any, None]:
    yield None
