from strenum import StrEnum


class TicketStatistics(object):
    def __init__(
        self,
        pending: int,
        unredeemed: int,
        unredeemedValue: int,
        redeemed: int,
        redeemedValue: int,
        losingTickets: int,
        winProportion: int,
        neglected: int,
        rejected: int,
        rejectedValue: str,
    ) -> None:
        self.pending = pending
        self.unredeemed = unredeemed
        self.unredeemedValue = unredeemedValue
        self.redeemed = redeemed
        self.redeemedValue = redeemedValue
        self.losingTickets = losingTickets
        self.winProportion = winProportion
        self.neglected = neglected
        self.rejected = rejected
        self.rejectedValue = rejectedValue


class TicketStatisticsKey(StrEnum):
    PENDING = ("pending",)
    UNREDEEMED = ("unredeemed",)
    UNREDEEMED_VALUE = ("unredeemedValue",)
    REDEEMED = ("redeemed",)
    REDEEMED_VALUE = ("redeemedValue",)
    LOSING_TICKETS = ("losingTickets",)
    WIN_PROPORTION = ("winProportion",)
    NEGLECTED = ("neglected",)
    REJECTED = ("rejected",)
    REJECTED_VALUE = "rejectedValue"
