from . import hello_func


class Hello:
    def hello_world(self) -> list[str]:
        res= list(hello_func.the_hello_func("world"))
        return "".join(res)
