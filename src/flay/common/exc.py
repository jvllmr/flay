import typing as t


class Serializable(t.Protocol):
    def __str__(self) -> str: ...


class FlayError(Exception):
    def __init__(
        self, msg: Serializable, rust_exc: Exception | None = None, *args: object
    ):
        self.msg = msg
        self.rust_exc = rust_exc
        super().__init__(*args)

    def __str__(self) -> str:
        if self.rust_exc:
            return f"{self.msg}: {self.rust_exc}"
        else:
            return str(self.msg)


class FlayFileNotFoundError(FlayError):
    pass


class FlayTreeshakeError(FlayError):
    pass
