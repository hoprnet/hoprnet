from decimal import Decimal

from api_lib.objects.response import (
    APIfield,
    APImetric,
    APIobject,
    JsonResponse,
    MetricResponse,
)

from .balance import Balance
from .channelstatus import ChannelStatus


@APIobject
class Addresses(JsonResponse):
    native: str = APIfield()


@APIobject
class Balances(JsonResponse):
    hopr: Balance = APIfield()
    native: Balance = APIfield()
    safe_native: Balance = APIfield("safeNative")
    safe_hopr: Balance = APIfield("safeHopr")
    safe_hopr_allowance: Balance = APIfield("safeHoprAllowance")


@APIobject
class Infos(JsonResponse):
    hopr_node_safe: bool = APIfield("hoprNodeSafe")


@APIobject
class ConnectedPeer(JsonResponse):
    address: str = APIfield()
    version: str = APIfield("reportedVersion")
    quality: float = APIfield()


@APIobject
class Channel(JsonResponse):
    balance: Balance = APIfield()
    channel_epoch: int = APIfield("channelEpoch")
    id: str = APIfield("channelId")
    closure_time: int = APIfield("closureTime")
    destination: str = APIfield()
    source: str = APIfield()
    status: ChannelStatus = APIfield()
    ticket_index: int = APIfield("ticketIndex")


@APIobject
class Ticket(JsonResponse):
    amount: Balance = APIfield()
    channel_epoch: int = APIfield("channelEpoch")
    channel_id: str = APIfield("channelId")
    index: int = APIfield()
    index_offset: int = APIfield("indexOffset")
    signature: str = APIfield()
    winn_prob: Decimal = APIfield("winProb")


@APIobject
class TicketPrice(JsonResponse):
    value: Balance = APIfield("price")


@APIobject
class TicketProbability(JsonResponse):
    _value: Decimal = APIfield("probability")

    @property
    def value(self) -> Decimal:
        return self._value.quantize(Decimal("1e-8"))

    @value.setter
    def value(self, value: Decimal):
        if not isinstance(value, Decimal):
            raise TypeError("Value must be a Decimal instance")
        self._value = value


@APIobject
class TicketStatistics(JsonResponse):
    neglected_value: Balance = APIfield("neglectedValue")
    redeemed_value: Balance = APIfield("redeemedValue")
    rejected_value: Balance = APIfield("rejectedValue")
    unredeemed_value: Balance = APIfield("unredeemedValue")
    winning_count: int = APIfield("winningCount")


@APIobject
class Configuration(JsonResponse):
    safe_address: str = APIfield("hopr/safe_module/safe_address")
    module_address: str = APIfield("hopr/safe_module/module_address")
    strategies: list[dict] = APIfield("hopr/strategy/strategies")


@APIobject
class OpenedChannel(JsonResponse):
    id: str = APIfield("channelId")


@APIobject
class Ping(JsonResponse):
    latency: float = APIfield()
    version: str = APIfield("reportedVersion")


@APIobject
class Session(JsonResponse):
    destination: str = APIfield()
    ip: str = APIfield()
    port: int = APIfield()
    protocol: str = APIfield()
    target: str = APIfield()
    forward_path: str = APIfield("forwardPath")
    return_path: str = APIfield("returnPath")
    mtu: int = APIfield()
    surb_len: int = APIfield("surbLen")
    active_clients: list[str] = APIfield("activeClients")


@APIobject
class SessionConfig(JsonResponse):
    response_buffer: str = APIfield("responseBuffer")
    max_surb_upstream: str = APIfield("maxSurbUpstream")


@APIobject
class Metrics(MetricResponse):
    hopr_tickets_incoming_statistics: dict = APImetric(["statistic"])


class Channels:
    def __init__(self, data: dict, category: str = "all"):
        self.all = []
        self.incoming = []
        self.outgoing = []

        setattr(self, category, [Channel(channel) for channel in data.get(category, [])])

    def __str__(self):
        return str(self.__dict__)

    def __repr__(self):
        return str(self)
