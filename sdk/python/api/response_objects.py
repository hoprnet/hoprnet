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
    keys = {"hopr": "hopr", "native": "native"}


class Alias(ApiResponseObject):
    keys = {"peer_id": "peerId"}


class AliasAddress(ApiResponseObject):
    keys = {"address": "address"}


class Balances(ApiResponseObject):
    keys = {
        "hopr": "hopr",
        "native": "native",
        "safe_native": "safeNative",
        "safe_hopr": "safeHopr",
        "safe_hopr_allowance": "safeHoprAllowance",
    }


class Infos(ApiResponseObject):
    keys = {"hopr_node_safe": "hoprNodeSafe"}


class ConnectedPeer(ApiResponseObject):
    keys = {"address": "peerAddress", "peer_id": "peerId", "version": "reportedVersion", "quality": "quality"}


class Channel(ApiResponseObject):
    keys = {
        "balance": "balance",
        "channel_epoch": "channelEpoch",
        "id": "channelId",
        "closure_time": "closureTime",
        "destination_address": "destinationAddress",
        "destination_peer_id": "destinationPeerId",
        "source_address": "sourceAddress",
        "source_peer_id": "sourcePeerId",
        "status": "status",
        "ticket_index": "ticketIndex",
    }

    def post_init(self):
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


class TicketPrice(ApiResponseObject):
    keys = {"value": "price"}


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


class Configuration(ApiResponseObject):
    keys = {"safe_address": "hopr/safe_module/safe_address", "module_address": "hopr/safe_module/module_address"}


class OpenedChannel(ApiResponseObject):
    keys = {"id": "channelId", "receipt": "transactionReceipt"}


class Ping(ApiResponseObject):
    keys = {"latency": "latency", "version": "reportedVersion"}


class Message(ApiResponseObject):
    keys = {"body": "body", "received_at": "receivedAt", "tag": "tag"}


class MessageSent(ApiResponseObject):
    keys = {"timestamp": "timestamp"}


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
        "ip": "ip",
        "port": "port",
        "protocol": "protocol",
        "target": "target",
    }
