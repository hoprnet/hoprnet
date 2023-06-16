import json
from typing import List
from .root_api_model import RootApiModel

from ..data.urls import Urls
from ..type.ticket import Ticket, TicketKey
from ..type.ticket_statistics import TicketStatistics, TicketStatisticsKey
from ..services.rest_api_service import RestApiService


class Tickets(RootApiModel):
    """
    Tickets API object wrapper with all useful methods to interact with tickets in the HOPR network.
    """

    def __init__(self) -> None:
        super().__init__()
        self.restService = RestApiService(self.get_auth_token())

    def get_all_tickets(self, nodeIndex: int) -> List[Ticket]:
        """
        Get all tickets earned from every channel by a node by relaying data packets.
        :nodeIndex: The node for which to fetch all the tickets.
        :return: List of Ticket objects with all the tickets.
        """
        data = self.restService.get_request(nodeIndex, Urls.TICKETS_LIST)
        tickets: List[Ticket] = []
        print("data = {}".format(json.dumps(data)))
        for dataTicket in data:
            # TODO Find a better, generic and shorter way to map this, json.loads() doesn't seem to help much since
            # we also have arrays returned from some of the API calls
            tickets.append(
                Ticket(
                    dataTicket[TicketKey.COUNTERPARTY.value],
                    dataTicket[TicketKey.CHALLENGE.value],
                    dataTicket[TicketKey.EPOCH.value],
                    dataTicket[TicketKey.INDEX.value],
                    dataTicket[TicketKey.AMOUNT.value],
                    dataTicket[TicketKey.WIN_PROB.value],
                    dataTicket[TicketKey.CHANNEL_EPOCH.value],
                    dataTicket[TicketKey.SIGNATURE.value],
                )
            )
        return tickets

    def redeem_all_tickets(self, nodeIndex: int) -> None:
        """
        Redeems all tickets from all the channels and exchanges them for Hopr tokens.
        Every ticket have a chance to be winning one, rewarding you with Hopr tokens.
        :nodeIndex:
        """
        url = self.get_rest_url(nodeIndex, Urls.TICKETS_REDEEM)
        payload = {}
        response = self.restService.post_request(url, payload)
        if response.status_code >= 400:
            self.handle_http_error(response)

    def get_all_tickets_statistics(self, nodeIndex: int, ticketsStatisticsKey: TicketStatisticsKey) -> int:
        """
        Get statistics regarding all your tickets. Node gets a ticket everytime it relays data packet in channel.
        Get a certain statistic value for the given node index.
        :nodeIndex: The node number for which the specific statistics is to be fetched.
        :ticketsStatisticsKey: The type of ticket statistics we want to fetch for the given node.
        :return: The desired statistic value as a number.
        """
        return int(self._get_tickets_statistics(nodeIndex, ticketsStatisticsKey.value))

    def get_tickets_statistics(self, nodeIndex: int) -> object:
        """ """
        data = self.restService.get_request(nodeIndex, Urls.TICKETS_STATISTICS)
        print("data = {}".format(json.dumps(data)))
        # TODO Find a better, generic and shorter way to map this, json.loads() doesn't seem to help much since
        # we also have arrays returned from some of the API calls, or have numbers defined as strings
        # (redeemed/unredeemedValue) so more granular interfacing is needed (at least for these more complicated types)
        ticketsStatistics = TicketStatistics(
            data[TicketStatisticsKey.PENDING.value],
            data[TicketStatisticsKey.UNREDEEMED.value],
            int(data[TicketStatisticsKey.UNREDEEMED_VALUE.value]),
            int(data[TicketStatisticsKey.REDEEMED.value]),
            data[TicketStatisticsKey.REDEEMED_VALUE.value],
            data[TicketStatisticsKey.LOSING_TICKETS.value],
            data[TicketStatisticsKey.WIN_PROPORTION.value],
            data[TicketStatisticsKey.NEGLECTED.value],
            data[TicketStatisticsKey.REJECTED.value],
            data[TicketStatisticsKey.REJECTED_VALUE.value],
        )
        return ticketsStatistics

    def _get_tickets_statistics(self, nodeIndex: int, statisticKey: str) -> str:
        """ """
        data = self.restService.get_request(nodeIndex, Urls.TICKETS_STATISTICS)
        return int(data[statisticKey])
