import asyncio
import base58
import base64
import logging
import nacl.utils
import nacl.bindings
import nacl.signing  # Ensure nacl.signing is imported correctly
import nacl.public
import random

from nacl.public import SealedBox  # Import SealedBox explicitly
from typing import Callable, Optional

import requests
from hoprd_sdk import ApiClient, Configuration
from hoprd_sdk.api import (
    AccountApi,
    AliasApi,
    ChannelsApi,
    MessagesApi,
    NetworkApi,
    NodeApi,
    PeersApi,
    SessionApi,
    TicketsApi,
)
from hoprd_sdk.models import (
    AliasDestinationBodyRequest,
    FundBodyRequest,
    GetMessageBodyRequest,
    OpenChannelBodyRequest,
    SendMessageBodyRequest,
    SessionClientRequest,
    SessionCloseClientRequest,
    TagQueryRequest,
    WithdrawBodyRequest,
)
from hoprd_sdk.rest import ApiException
from urllib3.exceptions import MaxRetryError


def getlogger():
    logger = logging.getLogger("hopr-api")
    logger.setLevel(logging.ERROR)

    return logger


log = getlogger()

MESSAGE_TAG = 1234


def seal_with_peerid(peer_id: str, plain_text: bytes, random_padding_len: int = 0) -> bytes:
    """
    This function takes a PeerID and plaintext data as inputs,
    extracts the Ed25519 public key corresponding to the PeerID,
    converts it to a Curve25519 key (for encryption),
    and returns a sealed box of the input plaintext encrypted using that public key.
    If specified, it also adds random padding with `@` character to the plaintext.
    This can be done to obscure the length of the plaintext.
    """
    try:
        # Step 1: Decode the PeerID from base58
        decoded_peer_id = base58.b58decode(peer_id)

        # Step 2: Extract the public key (skip the multicodec prefix)
        ed25519_pubkey = decoded_peer_id[6:]

        # Step 3: Convert the Ed25519 public key to a Curve25519 public key for encryption
        curve25519_pubkey = nacl.bindings.crypto_sign_ed25519_pk_to_curve25519(ed25519_pubkey)

        # Step 4: Create a PublicKey object from the Curve25519 public key bytes
        public_key = nacl.public.PublicKey(curve25519_pubkey)

        # Step 5: Create a sealed box for encryption using the public key
        sealed_box = SealedBox(public_key)

        # Step 6: Append random padding if random_padding_len is greater than 0
        plain_text += b"@" * random.randint(0, random_padding_len)

        # Step 7: Encrypt the plaintext using the sealed box
        encrypted_message = sealed_box.encrypt(plain_text)

        return encrypted_message
    except Exception as e:
        raise ValueError(f"seal failed: {str(e)}")


class HoprdAPI:
    """
    HOPRd API helper to handle exceptions and logging.
    """

    def __init__(self, url: str, token: str):
        self.configuration = Configuration()
        self.configuration.host = url
        self.configuration.api_key["X-Auth-Token"] = token

    def __call_api(self, obj: Callable[..., object], method: str, *args, **kwargs) -> tuple[bool, Optional[object]]:
        try:
            with ApiClient(self.configuration) as client:
                api_callback = getattr(obj(client), method)
                kwargs["async_req"] = True
                thread = api_callback(*args, **kwargs)
                response = thread.get()
                log.debug(
                    f"Calling {api_callback.__qualname__} with kwargs: {kwargs}, args: {args}, response: {response}"
                )
                return (True, response)
        except ApiException as e:
            log.debug(
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

    async def aliases_set_alias(self, alias: str, destination: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        body = AliasDestinationBodyRequest(alias, destination)
        status, _ = self.__call_api(AliasApi, "set_alias", body=body)
        return status

    async def aliases_remove_alias(self, alias: str):
        """
        Returns the aliases recognized by the node.
        :return: bool
        """
        status, _ = self.__call_api(AliasApi, "delete_alias", alias)
        return status

    async def addresses(self, address_type: str = "all"):
        """
        Returns the address of the node.
        :param: address: str = "hopr" | "native" | "all"
        :return: address: str | undefined
        """
        if address_type not in ["hopr", "native", "all"]:
            log.error(f"Invalid address type: {address_type}")
            return None
        if address_type == "all":
            address_type = ["hopr", "native"]
        if isinstance(address_type, str):
            address_type = [address_type]

        status, response = self.__call_api(AccountApi, "addresses")
        if not status:
            return None

        return_dict = {}
        for type in address_type:
            if not hasattr(response, type):
                log.error(f"No {address_type} returned from the API")
                return None
            return_dict[type] = getattr(response, type)

        return return_dict if len(return_dict) > 1 else return_dict[address_type[0]]

    async def balances(self):
        """
        Returns the balance of the node.
        :return: balances: dict | int
        """
        status, response = self.__call_api(AccountApi, "balances")
        return response if status else None

    async def open_channel(self, destination: str, amount: str):
        """
        Opens a channel with the given destination and amount.
        :param: destination: str
        :param: amount: str
        :return: channel id: str | undefined
        """
        body = OpenChannelBodyRequest(amount, destination=destination)

        status, response = self.__call_api(ChannelsApi, "open_channel", body=body)
        return response.channel_id if status else None

    async def channels_fund_channel(self, channel_id: str, amount: str):
        """
        Funds a given channel.
        :param: channel_id: str
        :param: amount: int
        :return: bool
        """
        body = FundBodyRequest(amount=amount)
        status, _ = self.__call_api(ChannelsApi, "fund_channel", body, channel_id)
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

        status, response = self.__call_api(
            ChannelsApi, "list_channels", full_topology="false", including_closed="false"
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
            ChannelsApi, "list_channels", full_topology="true", including_closed="true" if include_closed else "false"
        )
        return response if status else []

    async def ping(self, destination: str):
        """
        Pings the given destination and returns the measure.
        :param: destination: str
        :return: response: dict
        """
        _, response = self.__call_api(PeersApi, "ping_peer", destination)
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

    async def reset_tickets_statistics(self):
        """
        Resets the ticket statistics of the node.
        :return: bool
        """
        status, _ = self.__call_api(TicketsApi, "reset_ticket_statistics")
        return status

    async def send_message(self, destination: str, message: str, hops: list[str], tag: int = MESSAGE_TAG) -> bool:
        """
        Sends a message to the given destination.
        :param: destination: str
        :param: message: str
        :param: hops: list[str]
        :param: tag: int = 0x0320
        :return: bool
        """
        body = SendMessageBodyRequest(body=message, hops=None, path=hops, destination=destination, tag=tag)
        _, response = self.__call_api(MessagesApi, "send_message", body=body)
        return response

    async def messages_pop(self, tag: int = MESSAGE_TAG) -> bool:
        """
        Pop next message from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = TagQueryRequest(tag=tag)
        _, response = self.__call_api(MessagesApi, "pop", body=body)
        return response

    async def messages_pop_all(self, tag: int = MESSAGE_TAG) -> dict:
        """
        Pop all messages from the inbox
        :param: tag = 0x0320
        :return: dict
        """
        body = GetMessageBodyRequest(tag=tag)

        _, response = self.__call_api(MessagesApi, "pop_all", body=body)
        return response

    async def messages_peek(self, tag: int = MESSAGE_TAG) -> dict:
        """
        Peek next message from the inbox
        :param: tag = 0x0320
        :return: dict
        """

        body = TagQueryRequest(tag=tag)
        _, response = self.__call_api(MessagesApi, "peek", body=body)
        return response

    async def messages_peek_all(self, tag: int = MESSAGE_TAG, timestamp: int = 0) -> dict:
        """
        Peek all messages from the inbox
        :param: tag = 0x0320
        :return: dict
        """
        if not isinstance(timestamp, int):
            body = GetMessageBodyRequest(tag=tag)
        else:
            body = GetMessageBodyRequest(tag=tag, timestamp=timestamp)

        _, response = self.__call_api(MessagesApi, "peek_all", body=body)
        return response

    async def messages_pop_all(self, tag: int = MESSAGE_TAG) -> dict:
        """
        Pop all messages from the inbox
        :param: tag = 0x0320
        :return: dict
        """
        body = TagQueryRequest(tag=tag)
        _, response = self.__call_api(MessagesApi, "pop_all", body=body)
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
        _, response = self.__call_api(NetworkApi, "price")
        return int(response.price) if hasattr(response, "price") else None

    async def session_client(
        self,
        destination: str,
        path: str,
        protocol: str,
        target: str,
        listen_on: str = "127.0.0.1:0",
        capabilities=None,
        sealed_target=False,
        service=False,
    ):
        """
        Returns the port of the client session.
        :param destination: Peer ID of the session exit node.
        :param path: Routing options for the session.
        :param protocol: Transport protocol for the session (TCP/UDP).
        :param target: Destination for the session packets.
        :param listen_on: The host to listen on for input packets (default: "127.0.0.1:0")
        :param capabilities: Optional list of capabilities for the session (default: None)
        :param sealed_target: The target parameter will be encrypted (default: False)
        :param service: If set, the target must be an integer representing Exit node service (default: False)
        """
        actual_target = (
            {"Sealed": base64.b64encode(seal_with_peerid(destination, bytes(target, "utf-8"), 50)).decode("ascii")}
            if sealed_target
            else {"Service": int(target)}
            if service
            else {"Plain": target}
        )

        if capabilities is None:
            body = SessionClientRequest(destination=destination, path=path, target=actual_target, listen_host=listen_on)
        else:
            body = SessionClientRequest(
                destination=destination,
                path=path,
                target=actual_target,
                listen_host=listen_on,
                capabilities=capabilities,
            )

        _, response = self.__call_api(SessionApi, "create_client", body=body, protocol=protocol)
        return int(response.port) if hasattr(response, "port") else None

    async def session_list_clients(self, protocol: str):
        """
        Returns opened session listeners.
        :return: sessions: dict
        """
        _, response = self.__call_api(SessionApi, "list_clients", protocol=protocol)
        return response

    async def session_close_client(self, protocol: str, bound_port: int, bound_ip: str = "127.0.0.1"):
        """
        Closes a previously opened and bound session
        """
        body = SessionCloseClientRequest(listening_ip=bound_ip, port=bound_port)

        status, _ = self.__call_api(SessionApi, "close_client", body=body, protocol=protocol)
        return status

    async def ticket_min_win_prob(self):
        """
        Returns the minimum incoming ticket winning probability.
        :return: probability: float
        """
        _, response = self.__call_api(NetworkApi, "probability")
        return getattr(response, "probability", None)

    async def withdraw(self, amount: str, receipient: str, currency: str):
        """
        Withdraws the given amount of token (Native or HOPR) to the given receipient.
        :param: amount: str
        :param: receipient: str
        :param: currency: str
        :return:
        """
        body = WithdrawBodyRequest(receipient, amount, currency)
        status, response = self.__call_api(AccountApi, "withdraw", body=body)
        return status, response

    async def metrics(self):
        _, response = self.__call_api(NodeApi, "metrics")
        return response

    async def startedz(self, timeout: int = 20):
        """
        Checks if the node is started.
        """
        return await is_url_returning_200(f"{self.configuration.host}/startedz", timeout)

    async def readyz(self, timeout: int = 20):
        """
        Checks if the node is ready to accept connections.
        """
        return await is_url_returning_200(f"{self.configuration.host}/readyz", timeout)


async def is_url_returning_200(url, timeout):
    async def check_url():
        ready = False

        while not ready:
            try:
                ready = requests.get(url, timeout=0.3).status_code == 200
            except Exception:
                await asyncio.sleep(0.2)

        return ready

    try:
        return await asyncio.wait_for(check_url(), timeout=timeout)
    except Exception:
        return False
