from __future__ import annotations

from collections.abc import Iterable

from .domain import NotificationCommand, PriceTick, Rule, RuleDirection
from .idempotency import IdempotencyStore


def process_price_tick(
    *,
    tick: PriceTick,
    rules: Iterable[Rule],
    idempotency_store: IdempotencyStore,
) -> list[NotificationCommand]:
    commands: list[NotificationCommand] = []

    for rule in rules:
        if rule.symbol != tick.symbol:
            continue

        matched = (
            tick.price >= rule.threshold
            if rule.direction == RuleDirection.ABOVE
            else tick.price <= rule.threshold
        )
        if not matched:
            continue

        is_first = idempotency_store.register_once(
            symbol=tick.symbol,
            rule_id=rule.id,
            timestamp=tick.timestamp,
            bucket_seconds=rule.bucket_seconds,
        )
        if not is_first:
            continue

        commands.append(
            NotificationCommand(
                chat_id=rule.chat_id,
                text=(
                    f"{tick.symbol} hit {tick.price:.2f} from {tick.provider} "
                    f"(rule={rule.id}, threshold={rule.threshold:.2f}, direction={rule.direction.value})"
                ),
            )
        )

    return commands
