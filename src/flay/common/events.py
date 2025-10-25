import typing as t
from pydantic import BaseModel, ConfigDict
from abc import ABCMeta, abstractmethod


class Event(BaseModel):
    model_config = ConfigDict(frozen=True)


TEventType = t.TypeVar("TEventType", bound=Event)


class EventHandler(t.Generic[TEventType], metaclass=ABCMeta):
    @abstractmethod
    def on_event(self, event: TEventType) -> None: ...


class NoopEventHandler(t.Generic[TEventType], EventHandler[TEventType]):
    def on_event(self, event: TEventType) -> None:
        pass
