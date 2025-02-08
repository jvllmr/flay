from .hello_world import hello_world
from .hello_world import inner_hello_world_alias

def main() -> None:
    hello_world()
    inner_hello_world_alias.hello_world()
