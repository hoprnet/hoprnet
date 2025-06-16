from dataclasses import dataclass, field, fields
from decimal import Decimal

from .balance import Balance
from .channelstatus import ChannelStatus
from .conversion import convert_unit


class ApiResponseObject:
    def __init__(self, data: dict):
        for f in fields(self):
            path = f.metadata.get("path", f.name)
            v = data
            for subkey in path.split("/"):
                v = v.get(subkey, None)
                if v is None:
                    break

            if "type" in f.metadata:
                setattr(self, f.name, f.metadata["type"](v))
            else:
                setattr(self, f.name, convert_unit(v))

        self.post_init()

    def post_init(self):
        pass

    @property
    def is_null(self):
        return all(getattr(self, key) is None for key in [f.name for f in fields(self)])

    @property
    def as_dict(self) -> dict:
        return {key: getattr(self, key) for key in [f.name for f in fields(self)]}

    def __str__(self):
        return str(self.__dict__)

    def __repr__(self):
        return str(self)

    def __eq__(self, other):
        return all(getattr(self, key) == getattr(other, key) for key in [f.name for f in fields(self)])


@dataclass(init=False)
class Addresses(ApiResponseObject):
    native: str = field(metadata={"path": "native"})


@dataclass(init=False)
class Balances(ApiResponseObject):
    hopr: Balance = field(metadata={"path": "hopr", "type": Balance})
    native: Balance = field(metadata={"path": "native", "type": Balance})
    safe_native: Balance = field(metadata={"path": "safeNative", "type": Balance})
    safe_hopr: Balance = field(metadata={"path": "safeHopr", "type": Balance})
    safe_hopr_allowance: Balance = field(metadata={"path": "safeHoprAllowance", "type": Balance})


@dataclass(init=False)
class Infos(ApiResponseObject):
    hopr_node_safe: bool = field(metadata={"path": "hoprNodeSafe"})


@dataclass(init=False)
class ConnectedPeer(ApiResponseObject):
    address: str = field(metadata={"path": "address"})
    version: str = field(metadata={"path": "reportedVersion"})
    quality: str = field(metadata={"path": "quality"})


@dataclass(init=False)
class Channel(ApiResponseObject):
    balance: Balance = field(metadata={"path": "balance", "type": Balance})
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    id: str = field(metadata={"path": "channelId"})
    closure_time: int = field(metadata={"path": "closureTime"})
    destination: str = field(default="", metadata={"path": "destination"})
    source: str = field(default="", metadata={"path": "source"})
    status: ChannelStatus = field(metadata={"path": "status"})
    ticket_index: int = field(metadata={"path": "ticketIndex"})

    def post_init(self):
        self.status = ChannelStatus.fromString(self.status)


@dataclass(init=False)
class Ticket(ApiResponseObject):
    amount: Balance = field(metadata={"path": "amount", "type": Balance})
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    channel_id: str = field(metadata={"path": "channelId"})
    index: int = field(metadata={"path": "index"})
    index_offset: int = field(metadata={"path": "indexOffset"})
    signature: str = field(metadata={"path": "signature"})
    winn_prob: Decimal = field(metadata={"path": "winProb", "type": Decimal})


@dataclass(init=False)
class TicketPrice(ApiResponseObject):
    value: Balance = field(metadata={"path": "value", "type": Balance})


@dataclass(init=False)
class TicketProbability(ApiResponseObject):
    value: Decimal = field(metadata={"path": "value", "type": Decimal})


@dataclass(init=False)
class TicketStatistics(ApiResponseObject):
    neglected_value: Balance = field(metadata={"path": "neglectedValue", "type": Balance})
    redeemed_value: Balance = field(metadata={"path": "redeemedValue", "type": Balance})
    rejected_value: Balance = field(metadata={"path": "rejectedValue", "type": Balance})
    unredeemed_value: Balance = field(metadata={"path": "unredeemedValue", "type": Balance})
    winning_count: int = field(metadata={"path": "winningCount"})


@dataclass(init=False)
class Configuration(ApiResponseObject):
    safe_address: str = field(metadata={"path": "hopr/safe_module/safe_address"})
    module_address: str = field(metadata={"path": "hopr/safe_module/module_address"})


@dataclass(init=False)
class OpenedChannel(ApiResponseObject):
    id: str = field(metadata={"path": "channelId"})
    receipt: str = field(metadata={"path": "transactionReceipt"})


@dataclass(init=False)
class Ping(ApiResponseObject):
    latency: float = field(metadata={"path": "latency"})
    version: str = field(metadata={"path": "reportedVersion"})


@dataclass(init=False)
class Session(ApiResponseObject):
    destination: str = field(metadata={"path": "destination"})
    ip: str = field(metadata={"path": "ip"})
    port: int = field(metadata={"path": "port"})
    protocol: str = field(metadata={"path": "protocol"})
    target: str = field(metadata={"path": "target"})
    forward_path: str = field(metadata={"path": "forwardPath"})
    return_path: str = field(metadata={"path": "returnPath"})
    mtu: int = field(metadata={"path": "mtu"})


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
