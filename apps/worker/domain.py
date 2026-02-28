from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import Enum


class RuleDirection(str, Enum):
    ABOVE = "above"
    BELOW = "below"


@dataclass(frozen=True)
class PriceTick:
    symbol: str
    price: float
    timestamp: datetime
    provider: str


@dataclass(frozen=True)
class Rule:
    id: str
    symbol: str
    threshold: float
    direction: RuleDirection
    chat_id: str
    bucket_seconds: int = 60


@dataclass(frozen=True)
class NotificationCommand:
    chat_id: str
    text: str
