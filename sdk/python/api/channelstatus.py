from enum import Enum


class ChannelStatus(Enum):
    Open = "Open"
    PendingToClose = "PendingToClose"
    Closed = "Closed"

    @classmethod
    def _missing_(cls, value):
        if isinstance(value, str):
            for status in cls:
                if status.value == value:
                    return status
            return None
        return super()._missing_(value)

    @property
    def is_pending(self):
        return self == self.PendingToClose

    @property
    def is_open(self):
        return self == self.Open

    @property
    def is_closed(self):
        return self == self.Closed
