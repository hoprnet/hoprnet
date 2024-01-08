import logging

from hoprd_sdk import ApiClient, Configuration
from hoprd_sdk.api import AccountApi, AliasApi, ChannelsApi, MessagesApi, NodeApi, PeersApi, TicketsApi
from hoprd_sdk.models import AliasPeerId, FundRequest, OpenChannelRequest, TagQuery, SendMessageReq
from hoprd_sdk.rest import ApiException
from urllib3.exceptions import MaxRetryError


def getlogger():
    logger = logging.getLogger("hopr-api")
    logger.setLevel(logging.ERROR)

    return logger


log = getlogger()


MESSAGE_TAG = 1234


class HoprdAPI:
    """
    HOPRd API helper to handle exceptions and logging.
    """

    def __init__(self, url: str, token: str):
        self.configuration = Configuration()
        self.configuration.host = f"{url}"
        self.configuration.api_key["X-Auth-Token"] = token

    def __call_api(self, obj, method, *args, **kwargs):
        try:
            with ApiClient(self.configuration) as client:
                api_callback = obj(client).__getattribute__(method)
                kwargs["async_req"] = True
                thread = api_callback(*args, **kwargs)
                response = thread.get()
                log.debug(
                    f"Calling {api_callback.__qualname__} with kwargs: {kwargs}, args: {args}, response: {response}"
                )
                return (True, response)
        except ApiException as e:
            log.info(
                f"ApiException calling {api_callback.__qualname__} with kwargs: {kwargs}, args: {args}, error is: {e}"
            )
            return (False, None)
        except OSError:
            log.error(f"OSError calling {api_callback.__qualname__} with kwargs: {kwargs}, args: {args}:")
            return (False, None)
        except MaxRetryError:
            log.error(f"MaxRetryError calling {api_callback.__qualname__} with kwargs: {kwargs}, args: {args}")
            return (False, None)

    async def aliases_get_aliases(self):
        """
        DO NOT USE - this is useless, returns a pair {'alice': None, 'bob': None} that never gets set!

        Returns the aliases recognized by the node.
        :return: aliases: dict
        """
        _, response = self.__call_api(AliasApi, "aliases")
        return response

    async def aliases_get_alias(self, alias: str):
        """
        Returns the peer id recognized by the node.
        :return: peer_id: str
        """
        status, response = self.__call_api(AliasApi, "get_alias", alias)
        return response.peer_id if status else None

    async def aliases_set_alias(self, alias: str, peer_id: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        body = AliasPeerId(alias, peer_id)
        status, _ = self.__call_api(AliasApi, "set_alias", body=body)
        return status

    async def aliases_remove_alias(self, alias: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        status, _ = self.__call_api(AliasApi, "delete_alias", alias)
        return status

    async def balances(self):
        """
        Returns the balance of the node.
        :return: balances: dict | int
        """
        status, response = self.__call_api(AccountApi, "balances")
        return response if status else None

    async def open_channel(self, peer_address: str, amount: str):
        """
        Opens a channel with the given peer_address and amount.
        :param: peer_address: str
        :param: amount: str
        :return: channel id: str | undefined
        """
        body = OpenChannelRequest(amount, peer_address)

        status, response = self.__call_api(ChannelsApi, "open_channel", body=body)
        return response.channel_id if status else None

    async def channels_fund_channel(self, channel_id: str, amount: str):
        """
        Funds a given channel.
        :param: channel_id: str
        :param: amount: str
        :return: bool
        """
        body = FundRequest(amount=amount)
        status, _ = self.__call_api(ChannelsApi, "fund_channel", channel_id, body=body)
        return status

    async def close_channel(self, channel_id: str):
        """
        Closes a given channel.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "close_channel", channel_id)
        return status

    async def channel_redeem_tickets(self, channel_id: str):
        """
        Redeems tickets in a specific channel.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "redeem_tickets_in_channel", channel_id)
        return status

    async def incoming_channels(self, only_id: bool = False):
        """
        Returns all open incoming channels.
        :return: channels: list
        """

        status, response = self.__call_api(ChannelsApi, "list_channels", full_topology=False, including_closed=False)
        if status:
            if not hasattr(response, "incoming"):
                log.warning("Response does not contain `incoming`")
                return []

            if len(response.incoming) == 0:
                log.info("No incoming channels")
                return []

            if only_id:
                return [channel.id for channel in response.incoming]
            else:
                return response.incoming
        else:
            return []

    async def outgoing_channels(self, only_id: bool = False):
        """
        Returns all open outgoing channels.
        :return: channels: list
        """
        status, response = self.__call_api(ChannelsApi, "list_channels")
        if status:
            if not hasattr(response, "outgoing"):
                log.warning("Response does not contain `outgoing`")
                return []

            if len(response.outgoing) == 0:
                log.info("No outgoing channels")
                return []

            if only_id:
                return [channel.id for channel in response.outgoing]
            else:
                return response.outgoing
        else:
            return []

    async def get_channel(self, channel_id: str):
        """
        Returns the channel object.
        :param: channel_id: str
        :return: channel: response
        """
        _, response = self.__call_api(ChannelsApi, "show_channel", channel_id)
        return response

    async def channels_aggregate_tickets(self, channel_id: str):
        """
        Aggregate channel tickets.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "aggregate_tickets_in_channel", channel_id)
        return status

    async def channel_get_tickets(self, channel_id: str):
        """
        Returns all channel tickets.
        :param: channel_id: str
        :return: tickets: response
        """
        status, response = self.__call_api(ChannelsApi, "show_channel_tickets", channel_id)
        return response if status else []

    async def all_channels(self, include_closed: bool):
        """
        Returns all channels.
        :param: include_closed: bool
        :return: channels: list
        """
        status, response = self.__call_api(
            ChannelsApi, "list_channels", full_topology=True, including_closed=include_closed
        )
        return response if status else []

    async def ping(self, peer_id: str):
        """
        Pings the given peer_id and returns the measure.
        :param: peer_id: str
        :return: response: dict
        """
        _, response = self.__call_api(PeersApi, "ping_peer", peer_id)
        return response

    async def peers(self, params: list or str = "peer_id", status: str = "connected"):
        """
        Returns a list of peers.
        :param: param: list or str = "peer_id"
        :param: status: str = "connected"
        :param: quality: int = 0..1
        :return: peers: list
        """
        is_ok, response = self.__call_api(NodeApi, "peers")
        if is_ok:
            if not hasattr(response, status):
                log.error(f"No `{status}` returned from the API")
                return []

            if len(getattr(response, status)) == 0:
                log.info(f"No peer with status `{status}`")
                return []

            params = [params] if isinstance(params, str) else params
            for param in params:
                if not hasattr(getattr(response, status)[0], param):
                    log.error(f"No param `{param}` found for peers")
                    return []

            output_list = []
            for peer in getattr(response, status):
                output_list.append({param: getattr(peer, param) for param in params})

            return output_list
        else:
            return []

    async def get_tickets_statistics(self):
        """
        Returns the ticket statistics of the node.
        :return: statistics: dict
        """
        _, response = self.__call_api(TicketsApi, "show_ticket_statistics")
        return response

    async def send_message(self, destination: str, message: str, hops: list[str], tag: int = MESSAGE_TAG) -> bool:
        """
        Sends a message to the given destination.
        :param: destination: str
        :param: message: str
        :param: hops: list[str]
        :param: tag: int = 0x0320
        :return: bool
        """
        body = SendMessageReq(message, None, hops, destination, tag)
        status, _ = self.__call_api(MessagesApi, "send_message", body=body)
        return status

    async def messages_pop(self, tag: int = MESSAGE_TAG) -> bool:
        """
        Pop next message from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = TagQuery(tag=tag)
        _, response = self.__call_api(MessagesApi, "pop", body=body)
        return response

    async def messages_peek(self, tag: int = MESSAGE_TAG) -> dict:
        """
        Peek next message from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = TagQuery(tag=tag)
        _, response = self.__call_api(MessagesApi, "peek", body=body)
        return response

    async def messages_peek_all(self, tag: int = MESSAGE_TAG) -> dict:
        """
        Peek all messages from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = TagQuery(tag=tag)
        _, response = self.__call_api(MessagesApi, "peek_all", body=body)
        return response

    async def tickets_redeem(self):
        """
        Redeems all tickets.
        :return: bool
        """
        status, _ = self.__call_api(TicketsApi, "redeem_all_tickets")
        return status

    async def ticket_price(self):
        """
        Returns the ticket price in wei.
        :return: price: int
        """
        _, response = self.__call_api(TicketsApi, "price")
        return int(response.price) if hasattr(response, "price") else None
