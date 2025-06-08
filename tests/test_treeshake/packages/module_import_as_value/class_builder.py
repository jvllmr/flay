from . import setters
_DEFAULT_ON_SETATTR = setters.pipe("Hallo Welt!")
class ClassBuilder:
    def __init__(self) -> None:
        self.setters = _DEFAULT_ON_SETATTR
