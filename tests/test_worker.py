from datetime import datetime, timedelta, timezone

from apps.worker.domain import PriceTick, Rule, RuleDirection
from apps.worker.events import EventBus
from apps.worker.idempotency import IdempotencyStore
from apps.worker.providers import CompositeTickIngestor, PollingTickProvider, WebSocketTickProvider
from apps.worker.service import WorkerService
from apps.worker.telegram import TelegramNotifier
from apps.worker.use_cases import process_price_tick


def test_process_price_tick_idempotent_by_bucket() -> None:
    now = datetime(2026, 1, 1, tzinfo=timezone.utc)
    rule = Rule(
        id="r1",
        symbol="BTCUSDT",
        threshold=100.0,
        direction=RuleDirection.ABOVE,
        chat_id="chat-1",
        bucket_seconds=60,
    )
    store = IdempotencyStore()

    first = process_price_tick(
        tick=PriceTick("BTCUSDT", 101.0, now, "ws"),
        rules=[rule],
        idempotency_store=store,
    )
    second = process_price_tick(
        tick=PriceTick("BTCUSDT", 102.0, now + timedelta(seconds=20), "ws"),
        rules=[rule],
        idempotency_store=store,
    )
    third = process_price_tick(
        tick=PriceTick("BTCUSDT", 103.0, now + timedelta(seconds=61), "ws"),
        rules=[rule],
        idempotency_store=store,
    )

    assert len(first) == 1
    assert second == []
    assert len(third) == 1


def test_worker_uses_websocket_then_fallback_polling() -> None:
    ts = datetime(2026, 1, 1, tzinfo=timezone.utc)

    def broken_ws():
        raise RuntimeError("ws disconnected")
        yield  # pragma: no cover

    def polling_ticks():
        yield PriceTick("ETHUSDT", 2000.0, ts, "polling")

    sent: list[tuple[str, str]] = []
    notifier = TelegramNotifier(lambda chat_id, text: sent.append((chat_id, text)))

    ingestor = CompositeTickIngestor(
        WebSocketTickProvider(broken_ws),
        PollingTickProvider(polling_ticks),
    )

    service = WorkerService(
        ingestor=ingestor,
        event_bus=EventBus(),
        rule_loader=lambda symbol: [
            Rule(
                id="r-eth",
                symbol=symbol,
                threshold=1900.0,
                direction=RuleDirection.ABOVE,
                chat_id="chat-eth",
            )
        ],
        idempotency_store=IdempotencyStore(),
        notifier=notifier,
    )

    service.run_once()

    assert len(sent) == 1
    assert sent[0][0] == "chat-eth"
    assert "ETHUSDT hit" in sent[0][1]
