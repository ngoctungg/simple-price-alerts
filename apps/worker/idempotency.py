from __future__ import annotations

from datetime import datetime, timezone


class IdempotencyStore:
    """In-memory idempotency registry keyed by (symbol, rule_id, time_bucket)."""

    def __init__(self) -> None:
        self._keys: set[tuple[str, str, int]] = set()

    def _bucket(self, timestamp: datetime, bucket_seconds: int) -> int:
        utc_seconds = int(timestamp.astimezone(timezone.utc).timestamp())
        return utc_seconds // bucket_seconds

    def register_once(
        self,
        *,
        symbol: str,
        rule_id: str,
        timestamp: datetime,
        bucket_seconds: int,
    ) -> bool:
        key = (symbol, rule_id, self._bucket(timestamp, bucket_seconds))
        if key in self._keys:
            return False
        self._keys.add(key)
        return True
