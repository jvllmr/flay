from dataclasses import dataclass
from contextlib import contextmanager
import typing as t
@dataclass
class MyClass:
    pass



@contextmanager
def my_context_manager() -> t.Generator[None,t.Any, None]:
    yield None
