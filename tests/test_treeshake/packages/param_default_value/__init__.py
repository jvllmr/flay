import typing as t
import dataclasses

@dataclasses.dataclass
class DefaultPlaceholder:
    value: t.Any

def Default(value: t.Any) -> DefaultPlaceholder:
    return DefaultPlaceholder(value=value)


class DefaultUser:
    def __init__(self, value: t.Any = Default(42)) -> None:
        self.value = value


def main() -> None:
    user = DefaultUser()
    print(user.value)

if __name__ == "__main__":
    main()
