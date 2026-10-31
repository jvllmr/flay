import typing as t
from abc import ABCMeta, abstractmethod

from pydantic import BaseModel, ConfigDict


class Event(BaseModel):
    model_config = ConfigDict(frozen=True)


TEventType = t.TypeVar("TEventType", bound=Event)


class EventHandler(t.Generic[TEventType], metaclass=ABCMeta):
    @abstractmethod
    def on_event(self, event: TEventType) -> None: ...


class NoopEventHandler(
    EventHandler[TEventType],
    t.Generic[TEventType],
):
    def on_event(self, event: TEventType) -> None:
        pass


NOOP_EVENT_HANDLER = NoopEventHandler()
