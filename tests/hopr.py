import logging
from hoprd_sdk import Configuration, ApiClient
from hoprd_sdk.models import AliasesBody, MessagesBody, MessagesPopBody, ChannelsBody, ChannelidFundBody
from hoprd_sdk.rest import ApiException
from hoprd_sdk.api import AliasesApi, NodeApi, MessagesApi, AccountApi, ChannelsApi, PeersApi, TicketsApi
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
        self.configuration.host = f"{url}/api/v3"
        self.configuration.api_key["x-auth-token"] = token

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
        _, response = self.__call_api(AliasesApi, "aliases_get_aliases")
        return response

    async def aliases_get_alias(self, alias: str):
        """
        Returns the peer id recognized by the node.
        :return: peer_id: str
        """
        status, response = self.__call_api(AliasesApi, "aliases_get_alias", alias)
        return response.peer_id if status else None

    async def aliases_set_alias(self, alias: str, peer_id: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        body = AliasesBody(peer_id, alias)
        status, response = self.__call_api(AliasesApi, "aliases_set_alias", body=body)
        return status

    async def aliases_remove_alias(self, alias: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        status, _ = self.__call_api(AliasesApi, "aliases_remove_alias", alias)
        return status

    async def balances(self, type: str or list[str] = "all"):
        """
        Returns the balance of the node.
        :param: type: str =  "all" | "hopr" | "native" | "safe_native" | "safe_hopr"
        :return: balances: dict | int
        """
        all_types = ["hopr", "native", "safe_native", "safe_hopr"]
        assert type in all_types

        status, response = self.__call_api(AccountApi, "account_get_balances")
        if status:
            return int(getattr(response, type))
        else:
            return response

    async def open_channel(self, peer_address: str, amount: str):
        """
        Opens a channel with the given peer_address and amount.
        :param: peer_address: str
        :param: amount: str
        :return: channel id: str | undefined
        """
        body = ChannelsBody(peer_address, amount)

        status, response = self.__call_api(ChannelsApi, "channels_open_channel", body=body)
        return response.channel_id if status else None

    async def channels_fund_channel(self, channel_id: str, amount: str):
        """
        Funds a given channel.
        :param: channel_id: str
        :param: amount: str
        :return: bool
        """
        body = ChannelidFundBody(amount=amount)
        status, _ = self.__call_api(ChannelsApi, "channels_fund_channel", channel_id, body=body)
        return status

    async def close_channel(self, channel_id: str):
        """
        Closes a given channel.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "channels_close_channel", channel_id)
        return status

    async def channel_redeem_tickets(self, channel_id: str):
        """
        Redeems tickets in a specific channel.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "channels_redeem_tickets", channel_id)
        return status

    async def incoming_channels(self, only_id: bool = False):
        """
        Returns all open incoming channels.
        :return: channels: list
        """

        status, response = self.__call_api(
            ChannelsApi, "channels_get_channels", full_topology="false", including_closed="false"
        )
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
        status, response = self.__call_api(ChannelsApi, "channels_get_channels")
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
        _, response = self.__call_api(ChannelsApi, "channels_get_channel", channel_id)
        return response

    async def channels_aggregate_tickets(self, channel_id: str):
        """
        Aggregate channel tickets.
        :param: channel_id: str
        :return: bool
        """
        status, _ = self.__call_api(ChannelsApi, "channels_aggregate_tickets", channel_id)
        return status

    async def channel_get_tickets(self, channel_id: str):
        """
        Returns all channel tickets.
        :param: channel_id: str
        :return: tickets: response
        """
        status, response = self.__call_api(ChannelsApi, "channels_get_tickets", channel_id)
        return response if status else []

    async def all_channels(self, include_closed: bool):
        """
        Returns all channels.
        :param: include_closed: bool
        :return: channels: list
        """
        status, response = self.__call_api(
            ChannelsApi, "channels_get_channels", full_topology="true", including_closed=include_closed
        )
        return response if status else []

    async def ping(self, peer_id: str):
        """
        Pings the given peer_id and returns the measure.
        :param: peer_id: str
        :return: response: dict
        """
        _, response = self.__call_api(PeersApi, "peers_ping_peer", peer_id)
        return response

    async def peers(self, params: list or str = "peer_id", status: str = "connected"):
        """
        Returns a list of peers.
        :param: param: list or str = "peer_id"
        :param: status: str = "connected"
        :param: quality: int = 0..1
        :return: peers: list
        """
        is_ok, response = self.__call_api(NodeApi, "node_get_peers")
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
        _, response = self.__call_api(TicketsApi, "tickets_get_statistics")
        return response

    async def get_address(self, address: str):
        """
        Returns the address of the node.
        :param: address: str = "hopr" | "native"
        :return: address: str | undefined
        """
        status, response = self.__call_api(AccountApi, "account_get_address", address)
        if status:
            if not hasattr(response, address):
                log.error(f"No {address} returned from the API")
                return None

            return getattr(response, address)
        else:
            return None

    async def send_message(self, destination: str, message: str, hops: list[str], tag: int = MESSAGE_TAG) -> bool:
        """
        Sends a message to the given destination.
        :param: destination: str
        :param: message: str
        :param: hops: list[str]
        :param: tag: int = 0x0320
        :return: bool
        """
        body = MessagesBody(tag, message, destination, path=hops)
        status, _ = self.__call_api(MessagesApi, "messages_send_message", body=body)
        return status

    async def messages_pop(self, tag: int = MESSAGE_TAG) -> bool:
        """
        Pop next message from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = MessagesPopBody(tag=tag)
        _, response = self.__call_api(MessagesApi, "messages_pop_message", body=body)
        return response

    async def tickets_redeem(self):
        """
        Redeems all tickets.
        :return: bool
        """
        status, _ = self.__call_api(TicketsApi, "tickets_redeem_tickets")
        return status
