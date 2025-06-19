from dataclasses import dataclass, field, fields
from decimal import Decimal

from .balance import Balance
from .channelstatus import ChannelStatus


class ApiResponseObject:
    def __init__(self, data: dict):
        for f in fields(self):
            path = f.metadata.get("path", f.name)
            v = data
            for subkey in path.split("/"):
                v = v.get(subkey, None)
                if v is None:
                    break

            setattr(self, f.name, f.type(v))

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


class ApiMetricResponseObject(ApiResponseObject):
    def __init__(self, data: str):
        self.data = data.split("\n")

        for f in fields(self):
            values = {}
            labels = f.metadata.get("labels", [])

            for line in self.data:
                if not line.startswith(f.name):
                    continue

                value = line.split(" ")[-1]

                if len(labels) == 0:
                    setattr(self, f.name, f.type(value) + getattr(self, f.name, 0))
                else:
                    labels_values = {
                        pair.split("=")[0].strip('"'): pair.split("=")[1].strip('"')
                        for pair in line.split("{")[1].split("}")[0].split(",")
                    }

                    dict_path = [labels_values[label] for label in labels]
                    current = values

                    for part in dict_path[:-1]:
                        if part not in current:
                            current[part] = {}
                        current = current[part]
                    if dict_path[-1] not in current:
                        current[dict_path[-1]] = Decimal("0")
                    current[dict_path[-1]] += Decimal(value)

            if len(labels) > 0:
                setattr(self, f.name, f.type(values))


@dataclass(init=False)
class Addresses(ApiResponseObject):
    native: str = field()


@dataclass(init=False)
class Balances(ApiResponseObject):
    hopr: Balance = field()
    native: Balance = field()
    safe_native: Balance = field(metadata={"path": "safeNative"})
    safe_hopr: Balance = field(metadata={"path": "safeHopr"})
    safe_hopr_allowance: Balance = field(metadata={"path": "safeHoprAllowance"})


@dataclass(init=False)
class Infos(ApiResponseObject):
    hopr_node_safe: bool = field(metadata={"path": "hoprNodeSafe"})


@dataclass(init=False)
class ConnectedPeer(ApiResponseObject):
    address: str = field()
    version: str = field(metadata={"path": "reportedVersion"})
    quality: str = field()


@dataclass(init=False)
class Channel(ApiResponseObject):
    balance: Balance = field()
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    id: str = field(metadata={"path": "channelId"})
    closure_time: int = field(metadata={"path": "closureTime"})
    destination: str = field()
    source: str = field()
    status: ChannelStatus = field()
    ticket_index: int = field(metadata={"path": "ticketIndex"})


@dataclass(init=False)
class Ticket(ApiResponseObject):
    amount: Balance = field()
    channel_epoch: int = field(metadata={"path": "channelEpoch"})
    channel_id: str = field(metadata={"path": "channelId"})
    index: int = field()
    index_offset: int = field(metadata={"path": "indexOffset"})
    signature: str = field()
    winn_prob: Decimal = field(metadata={"path": "winProb"})


@dataclass(init=False)
class TicketPrice(ApiResponseObject):
    value: Balance = field()


@dataclass(init=False)
class TicketProbability(ApiResponseObject):
    value: Decimal = field()


@dataclass(init=False)
class TicketStatistics(ApiResponseObject):
    neglected_value: Balance = field(metadata={"path": "neglectedValue"})
    redeemed_value: Balance = field(metadata={"path": "redeemedValue"})
    rejected_value: Balance = field(metadata={"path": "rejectedValue"})
    unredeemed_value: Balance = field(metadata={"path": "unredeemedValue"})
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
    latency: float = field()
    version: str = field(metadata={"path": "reportedVersion"})


@dataclass(init=False)
class Session(ApiResponseObject):
    destination: str = field()
    ip: str = field()
    port: int = field()
    protocol: str = field()
    target: str = field()
    forward_path: str = field(metadata={"path": "forwardPath"})
    return_path: str = field(metadata={"path": "returnPath"})
    mtu: int = field()


@dataclass(init=False)
class Metrics(ApiMetricResponseObject):
    hopr_tickets_incoming_statistics: dict = field(metadata={"labels": ["statistic"]})


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
