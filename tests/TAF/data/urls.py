from strenum import StrEnum


class Urls(StrEnum):
    ACCOUNT_WITHDRAW = ("account/withdraw",)
    ACCOUNT_BALANCES = ("account/balances",)
    ACCOUNT_ADDRESSES = ("account/addresses",)

    ALIASES_LIST = ("aliases",)
    ALIASES_CREATE = ("aliases",)
    ALIASES_PEER_ID = ("aliases/{alias}",)
    ALIASES_DELETE = ("aliases/{alias}",)

    CHANNELS_FUND_PAYMENT_CHANNEL = ("fundmulti",)
    CHANNELS_OPEN_PAYMENT_CHANNEL = ("channels",)
    CHANNELS_ACTIVE_CHANNEL_LIST = ("channels/",)
    CHANNELS_REDEEM_TICKETS = ("channels/{peerId}/tickets/redeem",)
    CHANNELS_TICKETS_LIST = ("channels/{peerId}/tickets",)
    CHANNELS_CLOSE_OPENED_CHANNEL = ("channels/{peerId}/{direction}",)
    CHANNELS_CHANNEL_INFO = ("channels/{peerId}/{direction}",)

    MESSAGES_INCOMING_MESSAGEs = ("messages/websocket",)
    MESSAGES_SIGN = ("messages/sign",)
    MESSAGES_SEND = ("messages",)

    NODE_VERSION = ("node/version",)
    NODE_PING = ("nodeping",)
    NODE_PEER_LIST = ("node/peers",)
    NODE_METRICS = ("node/metrics",)
    NODE_INFO = ("node/info",)
    NODE_ENTRY_NODES = ("node/entryNodes",)

    PEER_INFO = ("peerInfo",)

    SETTINGS_GET_NODE_SETTINGS = ("settings",)
    SETTINGS_UPDATE_NODE_SETTING_VALUE = ("settings/{setting}",)

    TICKETS_STATISTICS = ("tickets/statistics",)
    TICKETS_REDEEM = ("tickets/redeem",)
    TICKETS_LIST = ("tickets",)

    TOKENS_CREATE_NEW_AUTH_TOKEN = ("tokens",)
    TOKENS_FULL_TOKEN_INFO = ("token",)
    TOKENS_DELETE_TOKEN = "tokens/{id}"
