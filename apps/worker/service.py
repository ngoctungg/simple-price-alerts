from __future__ import annotations

from collections.abc import Callable, Iterable

from .domain import Rule
from .events import EventBus, PriceTickReceived
from .idempotency import IdempotencyStore
from .providers import CompositeTickIngestor
from .telegram import TelegramNotifier
from .use_cases import process_price_tick


class WorkerService:
    def __init__(
        self,
        *,
        ingestor: CompositeTickIngestor,
        event_bus: EventBus,
        rule_loader: Callable[[str], Iterable[Rule]],
        idempotency_store: IdempotencyStore,
        notifier: TelegramNotifier,
    ) -> None:
        self._ingestor = ingestor
        self._event_bus = event_bus
        self._rule_loader = rule_loader
        self._idempotency_store = idempotency_store
        self._notifier = notifier
        self._event_bus.subscribe(PriceTickReceived, self._handle_price_tick_received)

    def run_once(self) -> None:
        for tick in self._ingestor.stream():
            self._event_bus.publish(PriceTickReceived(tick=tick))

    def _handle_price_tick_received(self, event: PriceTickReceived) -> None:
        rules = self._rule_loader(event.tick.symbol)
        commands = process_price_tick(
            tick=event.tick,
            rules=rules,
            idempotency_store=self._idempotency_store,
        )
        for command in commands:
            self._notifier.send(command)
