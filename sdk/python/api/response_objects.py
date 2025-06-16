from dataclasses import dataclass, field, fields
from decimal import Decimal
from typing import Any

from .channelstatus import ChannelStatus


def _convert(value: Any):
    if value is None:
        return None

    if isinstance(value, int) or isinstance(value, float):
        return value

    if isinstance(value, str):
        try:
            float_value = float(value)
        except ValueError:
            return value

        if "." in value:
            return float_value
        else:
            return int(value)

    return value


def _parse_balance_string(balance_str) -> Decimal:
    """Parse a balance string with currency units and return the amount as a big decimal"""
    try:
        return Decimal(balance_str.split()[0])
    except (ValueError, AttributeError, IndexError) as e:
        raise ValueError(f"Failed to parse balance string: {balance_str}") from e


class ApiResponseObject:
    def __init__(self, data: dict):
        for f in fields(self):
            path = f.metadata.get("path", f.name)
            v = data
            for subkey in path.split("/"):
                v = v.get(subkey, None)
                if v is None:
                    break
            setattr(self, f.name, _convert(v))
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
        return all(
            getattr(self, key) == getattr(other, key) for key in [f.name for f in fields(self)]
        )


@dataclass(init=False)
class Addresses(ApiResponseObject):
    native: str = field(metadata={"path": "native"})


@dataclass(init=False)
class Balances(ApiResponseObject):
    hopr: Decimal = field(metadata={"path": "hopr"})
    native: Decimal = field(metadata={"path": "native"})
    safe_native: Decimal = field(metadata={"path": "safeNative"})
    safe_hopr: Decimal = field(metadata={"path": "safeHopr"})
    safe_hopr_allowance: Decimal = field(metadata={"path": "safeHoprAllowance"})
    
    def post_init(self):
        self.hopr = _parse_balance_string(self.hopr)
        self.native = _parse_balance_string(self.native)
        self.safe_native = _parse_balance_string(self.safe_native)
        self.safe_hopr = _parse_balance_string(self.safe_hopr)
        self.safe_hopr_allowance = _parse_balance_string(self.safe_hopr_allowance)


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
    balance: Decimal = field(metadata={"path": "balance"})
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    id: str = field(metadata={"path": "channelId"})
    closure_time: int = field(metadata={"path": "closureTime"})
    destination: str = field(default="", metadata={"path": "destination"})
    source: str = field(default="", metadata={"path": "source"})
    status: ChannelStatus = field(metadata={"path": "status"})
    ticket_index: int = field(metadata={"path": "ticketIndex"})

    def post_init(self):
        self.balance = _parse_balance_string(self.balance)
        self.status = ChannelStatus.fromString(self.status)


@dataclass(init=False)
class Ticket(ApiResponseObject):
    amount: Decimal = field(metadata={"path": "amount"})
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    channel_id: str = field(metadata={"path": "channelId"})
    index: int = field(metadata={"path": "index"})
    index_offset: int = field(metadata={"path": "indexOffset"})
    signature: str = field(metadata={"path": "signature"})
    winn_prob: float = field(metadata={"path": "winProb"})

    def post_init(self):
        self.amount = _parse_balance_string(self.amount)


@dataclass(init=False)
class TicketPrice(ApiResponseObject):
    value: Decimal = field(metadata={"path": "value"})

    def post_init(self):
        self.value = _parse_balance_string(self.value)


@dataclass(init=False)
class TicketProbability(ApiResponseObject):
    value: Decimal = field(metadata={"path": "value"})

    def post_init(self):
        self.value = Decimal(self.value)


@dataclass(init=False)
class TicketStatistics(ApiResponseObject):
    neglected_value: Decimal = field(metadata={"path": "neglectedValue"})
    redeemed_value: Decimal = field(metadata={"path": "redeemedValue"})
    rejected_value: Decimal = field(metadata={"path": "rejectedValue"})
    unredeemed_value: Decimal = field(metadata={"path": "unredeemedValue"})
    winning_count: int = field(metadata={"path": "winningCount"})

    def post_init(self):
        self.rejected_value = _parse_balance_string(self.rejected_value)
        self.neglected_value = _parse_balance_string(self.neglected_value)
        self.redeemed_value = _parse_balance_string(self.redeemed_value)
        self.unredeemed_value = _parse_balance_string(self.unredeemed_value)


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

        setattr(self, category, [Channel(channel)
                for channel in data.get(category, [])])

    def __str__(self):
        return str(self.__dict__)

    def __repr__(self):
        return str(self)
