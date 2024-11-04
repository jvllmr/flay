class FlayError(Exception):
    pass


class FlayFileNotFoundError(FlayError):
    pass


class ParsingError(FlayError):
    pass
