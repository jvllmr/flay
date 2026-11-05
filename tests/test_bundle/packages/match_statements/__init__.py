class Point:
    def __init__(self, x=0, y=0):
        self.x = x
        self.y = y


def describe(value):
    match value:
        case bytes() | bytearray():
            return "binary"
        case str():
            return "text"
        case Point(x=px, y=py):
            return f"point {px},{py}"
        case [first, *rest]:
            return "sequence"
        case {"kind": kind, **others}:
            return f"mapping {kind}"
        case 1 | 2 | 3 as small:
            return f"small {small}"
        case n if n > 10:
            return "big"
        case None:
            return "none"
        case _:
            return "other"
