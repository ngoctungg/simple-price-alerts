from __future__ import annotations

from collections.abc import Callable

from .domain import NotificationCommand


class TelegramNotifier:
    def __init__(self, send_message: Callable[[str, str], None]) -> None:
        self._send_message = send_message

    def send(self, command: NotificationCommand) -> None:
        self._send_message(command.chat_id, command.text)
