from strenum import StrEnum


class Ticket(object):
    def __init__(
        self,
        counterparty: str,
        challenge: str,
        epoch: str,
        index: str,
        amount: str,
        winProb: str,
        channelEpoch: str,
        signature: str,
    ) -> None:
        self.counterparty = counterparty
        self.challenge = challenge
        self.epoch = epoch
        self.index = index
        self.amount = amount
        self.winProb = winProb
        self.channelEpoch = channelEpoch
        self.signature = signature


class TicketKey(StrEnum):
    COUNTERPARTY = ("counterparty",)
    CHALLENGE = ("challenge",)
    EPOCH = ("epoch",)
    INDEX = ("index",)
    AMOUNT = ("amount",)
    WIN_PROB = ("winProb",)
    CHANNEL_EPOCH = ("channelEpoch",)
    SIGNATURE = "signature"
