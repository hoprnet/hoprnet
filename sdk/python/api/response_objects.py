import logging
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


def _parse_balance_string(balance_str):
    """Parse a balance string with currency units and return the amount as a big decimal"""
    try:
        return Decimal(balance_str.split()[0])
    except (ValueError, AttributeError, IndexError) as e:
        raise ValueError(f"Failed to parse balance string: {balance_str}") from e


class ApiResponseObject:
    def __init__(self, data: dict):
        for key, value in self.keys.items():
            v = data
            for subkey in value.split("/"):
                v = v.get(subkey, None)
                if v is None:
                    continue

            setattr(self, key, _convert(v))

        self.post_init()

    def post_init(self):
        pass

    @property
    def is_null(self):
        return all(getattr(self, key) is None for key in self.keys.keys())

    @property
    def as_dict(self) -> dict:
        return {key: getattr(self, key) for key in self.keys.keys()}

    def __str__(self):
        return str(self.__dict__)

    def __repr__(self):
        return str(self)

    def __eq__(self, other):
        return all(getattr(self, key) == getattr(other, key) for key in self.keys.keys())


class Addresses(ApiResponseObject):
    keys = {"native": "native"}


class Balances(ApiResponseObject):
    keys = {
        "hopr": "hopr",
        "native": "native",
        "safe_native": "safeNative",
        "safe_hopr": "safeHopr",
        "safe_hopr_allowance": "safeHoprAllowance",
    }

    def post_init(self):
        self.hopr = _parse_balance_string(self.hopr)
        self.native = _parse_balance_string(self.native)
        self.safe_native = _parse_balance_string(self.safe_native)
        self.safe_hopr = _parse_balance_string(self.safe_hopr)
        self.safe_hopr_allowance = _parse_balance_string(self.safe_hopr_allowance)


class Infos(ApiResponseObject):
    keys = {"hopr_node_safe": "hoprNodeSafe"}


class ConnectedPeer(ApiResponseObject):
    keys = {"address": "address", "version": "reportedVersion", "quality": "quality"}


class Channel(ApiResponseObject):
    keys = {
        "balance": "balance",
        "channel_epoch": "channelEpoch",
        "id": "channelId",
        "closure_time": "closureTime",
        "destination": "destination",
        "source": "source",
        "status": "status",
        "ticket_index": "ticketIndex",
    }

    def post_init(self):
        self.balance = _parse_balance_string(self.balance)
        self.status = ChannelStatus.fromString(self.status)


class Ticket(ApiResponseObject):
    keys = {
        "amount": "amount",
        "channel_epoch": "channelEpoch",
        "channel_id": "channelId",
        "index": "index",
        "index_offset": "indexOffset",
        "signature": "signature",
        "winn_prob": "winProb",
    }

    def post_init(self):
        self.amount = _parse_balance_string(self.amount)


class TicketPrice(ApiResponseObject):
    keys = {"value": "price"}

    def post_init(self):
        self.value = _parse_balance_string(self.value)


class TicketProbability(ApiResponseObject):
    keys = {"value": "probability"}

    def post_init(self):
        self.value = float(self.value)


class TicketStatistics(ApiResponseObject):
    keys = {
        "neglected_value": "neglectedValue",
        "redeemed_value": "redeemedValue",
        "rejected_value": "rejectedValue",
        "unredeemed_value": "unredeemedValue",
        "winning_count": "winningCount",
    }

    def post_init(self):
        self.rejected_value = _parse_balance_string(self.rejected_value)
        self.neglected_value = _parse_balance_string(self.neglected_value)
        self.redeemed_value = _parse_balance_string(self.redeemed_value)
        self.unredeemed_value = _parse_balance_string(self.unredeemed_value)


class Configuration(ApiResponseObject):
    keys = {"safe_address": "hopr/safe_module/safe_address", "module_address": "hopr/safe_module/module_address"}


class OpenedChannel(ApiResponseObject):
    keys = {"id": "channelId", "receipt": "transactionReceipt"}


class Ping(ApiResponseObject):
    keys = {"latency": "latency", "version": "reportedVersion"}


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


class Session(ApiResponseObject):
    keys = {
        "destination": "destination",
        "ip": "ip",
        "port": "port",
        "protocol": "protocol",
        "target": "target",
        "forward_path": "forwardPath",
        "return_path": "returnPath",
        "mtu": "mtu",
    }
