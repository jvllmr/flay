from ...hello_world import hello_world
from ... import hello_world as hello_world_module

def goodbye_world() -> None:
    print("Goodbye, world!")

__all__ = ["hello_world", "hello_world_module", "goodbye_world"]
