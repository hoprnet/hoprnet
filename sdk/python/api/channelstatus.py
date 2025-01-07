from enum import Enum


class ChannelStatus(Enum):
    Open = "Open"
    PendingToClose = "PendingToClose"
    Closed = "Closed"

    @property
    def is_pending(self):
        return self == self.PendingToClose

    @property
    def is_open(self):
        return self == self.Open

    @property
    def is_closed(self):
        return self == self.Closed

    @classmethod
    def fromString(cls, value: str):
        for status in cls:
            if status.value == value:
                return status

        return None
