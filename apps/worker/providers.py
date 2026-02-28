from __future__ import annotations

from collections.abc import Callable, Iterable
from typing import Protocol

from .domain import PriceTick


class TickProvider(Protocol):
    def stream(self) -> Iterable[PriceTick]:
        ...


class WebSocketTickProvider:
    """Primary provider: yields ticks from a websocket source."""

    def __init__(self, websocket_client: Callable[[], Iterable[PriceTick]]) -> None:
        self._websocket_client = websocket_client

    def stream(self) -> Iterable[PriceTick]:
        return self._websocket_client()


class PollingTickProvider:
    """Fallback provider: yields ticks from polling source."""

    def __init__(self, polling_client: Callable[[], Iterable[PriceTick]]) -> None:
        self._polling_client = polling_client

    def stream(self) -> Iterable[PriceTick]:
        return self._polling_client()


class CompositeTickIngestor:
    """Use websocket and gracefully fallback to polling if websocket fails."""

    def __init__(self, websocket_provider: TickProvider, polling_provider: TickProvider) -> None:
        self._websocket_provider = websocket_provider
        self._polling_provider = polling_provider

    def stream(self) -> Iterable[PriceTick]:
        try:
            yield from self._websocket_provider.stream()
        except Exception:
            yield from self._polling_provider.stream()
