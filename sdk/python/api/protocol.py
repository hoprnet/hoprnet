from enum import Enum


class ProcolTemplate:
    def __init__(self, retransmit: bool, segment: bool):
        self.retransmit = retransmit
        self.segment = segment


class TCPProtocol(ProcolTemplate):
    def __init__(self):
        super().__init__(retransmit=True, segment=True)


class UDPProtocol(ProcolTemplate):
    def __init__(self):
        super().__init__(retransmit=False, segment=False)


class Protocol(Enum):
    TCP = TCPProtocol()
    UDP = UDPProtocol()

    @classmethod
    def fromString(cls, protocol: str):
        try:
            return getattr(cls, protocol.upper())
        except AttributeError:
            raise ValueError(
                f"Invalid protocol: {protocol}. Valid values are: {[p.name for p in cls]}"
            )

    @property
    def segment(self):
        return self.value.segment

    @property
    def retransmit(self):
        return self.value.retransmit

    def __eq__(self, other):
        if isinstance(other, str):
            return other == self.name

        return self.name == other.name
