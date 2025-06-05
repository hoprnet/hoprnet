import asyncio
import base64
import logging
import random
from decimal import Decimal, ROUND_UP
from typing import Optional

import aiohttp
import base58
import nacl.bindings
import nacl.public
import nacl.signing  # Ensure nacl.signing is imported correctly
import nacl.utils
from nacl.public import SealedBox  # Import SealedBox explicitly


from .channelstatus import ChannelDirection, ChannelStatus
from .http_method import HTTPMethod
from .protocol import Protocol
from .request_objects import (
    ApiRequestObject,
    CreateSessionBody,
    FundChannelBody,
    GetChannelsBody,
    OpenChannelBody,
    CloseChannelsBody,
    SessionCapabilitiesBody,
    WithdrawBody,
)
from .response_objects import (
    Addresses,
    Balances,
    Channel,
    Channels,
    Configuration,
    ConnectedPeer,
    Infos,
    OpenedChannel,
    Ping,
    Session,
    Ticket,
    TicketPrice,
    TicketProbability,
    TicketStatistics,
)

MESSAGE_TAG = 0x1245


def getlogger():
    logger = logging.getLogger("hopr-api")
    logger.setLevel(logging.ERROR)

    return logger


log = getlogger()


def seal_with_address(address: str, plain_text: bytes, random_padding_len: int = 0) -> bytes:
    """
    This function takes an Address and plaintext data as inputs,
    extracts the Ed25519 public key corresponding to the Address,
    converts it to a Curve25519 key (for encryption),
    and returns a sealed box of the input plaintext encrypted using that public key.
    If specified, it also adds random padding with `@` character to the plaintext.
    This can be done to obscure the length of the plaintext.
    """
    try:
        # Step 1: Decode the Address from base58
        decoded_address = base58.b58decode(address)

        # Step 2: Extract the public key (skip the multicodec prefix)
        ed25519_pubkey = decoded_address[6:]

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
        self.host = url
        self.headers = {"Authorization": f"Bearer {token}"}
        self.prefix = "/api/v4/"

    @property
    def log_prefix(cls) -> str:
        return "api"

    async def __call(self, method: HTTPMethod, endpoint: str, data: ApiRequestObject = None, use_api_path: bool = True):
        try:
            headers = {"Content-Type": "application/json"}
            async with aiohttp.ClientSession(headers=self.headers) as s:
                url = f"{self.host}{self.prefix if use_api_path else '/'}{endpoint}"
                logging.debug(f"Calling {method.value} {url}")
                async with getattr(s, method.value)(
                    url=url,
                    json={} if data is None else data.as_dict,
                    headers=headers,
                ) as res:
                    try:
                        data = await res.json()
                    except Exception:
                        data = await res.text()

                    return (res.status // 200) == 1, data

        except OSError as e:
            logging.error(f"OSError calling {method.value} {endpoint}: {e}")

        except Exception as e:
            logging.error(f"Exception calling {method.value} {endpoint}. error is: {e}")

        return (False, None)

    async def __call_api(
        self,
        method: HTTPMethod,
        endpoint: str,
        data: ApiRequestObject = None,
        timeout: int = 60,
        use_api_path: bool = True,
    ) -> tuple[bool, Optional[object]]:
        try:
            return await asyncio.wait_for(
                asyncio.create_task(self.__call(method, endpoint, data, use_api_path)),
                timeout=timeout,
            )

        except asyncio.TimeoutError:
            logging.error(f"TimeoutError calling {method} {endpoint}")
            return (False, None)

    async def balances(self) -> Optional[Balances]:
        """
        Returns the balance of the node.
        :return: balances: Balances | undefined
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "account/balances")
        return Balances(response) if is_ok else None

    async def open_channel(self, destination: str, amount: Decimal) -> Optional[OpenedChannel]:
        """
        Opens a channel with the given peer_address and amount.
        :param: peer_address: str
        :param: amount: Decimal
        :return: channel id: str | undefined
        """
        data = OpenChannelBody(amount, destination)

        is_ok, response = await self.__call_api(HTTPMethod.POST, "channels", data)
        return OpenedChannel(response) if is_ok else None

    async def fund_channel(self, channel_id: str, amount: Decimal) -> bool:
        """
        Funds a given channel.
        :param: channel_id: str
        :param: amount: float
        :return: bool
        """
        data = FundChannelBody(amount)

        is_ok, response = await self.__call_api(HTTPMethod.POST, f"channels/{channel_id}/fund", data)
        return is_ok

    async def close_channel(self, channel_id: str) -> bool:
        """
        Closes a given channel.
        :param: channel_id: str
        :return: bool
        """
        is_ok, _ = await self.__call_api(HTTPMethod.DELETE, f"channels/{channel_id}")
        return is_ok

    async def close_channels(self, direction: ChannelDirection, status: ChannelStatus) -> bool:
        """
        Closes multiple channels at once.
        """
        data = CloseChannelsBody(direction.value, status.value)
        is_ok, _ = await self.__call_api(HTTPMethod.DELETE, "channels", data)
        return is_ok

    async def channel_redeem_tickets(self, channel_id: str) -> bool:
        """
        Redeems tickets in a specific channel.
        :param: channel_id: str
        :return: bool
        """
        is_ok, _ = await self.__call_api(HTTPMethod.POST, f"channels/{channel_id}/tickets/redeem")
        return is_ok

    async def all_channels(self, include_closed: bool) -> Optional[Channels]:
        """
        Returns all channels.
        :return: channels: list
        """
        params = GetChannelsBody("true", "true" if include_closed else "false")

        is_ok, response = await self.__call_api(HTTPMethod.GET, f"channels?{params.as_header_string}")
        return Channels(response, "all") if is_ok else None

    async def incoming_channels(self, include_closed: bool = False):
        """
        Returns all incoming channels.
        :return: channels: list
        """
        params = GetChannelsBody("false", "true" if include_closed else "false")

        is_ok, response = await self.__call_api(HTTPMethod.GET, f"channels?{params.as_header_string}")
        return Channels(response, "incoming") if is_ok else None

    async def outgoing_channels(self, include_closed: bool = False):
        """
        Returns all outgoing channels.
        :return: channels: list
        """
        params = GetChannelsBody("false", "true" if include_closed else "false")

        is_ok, response = await self.__call_api(HTTPMethod.GET, f"channels?{params.as_header_string}")
        return Channels(response, "outgoing") if is_ok else None

    async def get_channel(self, channel_id: str) -> Optional[Channel]:
        """
        Returns the channel object.
        :param: channel_id: str
        :return: channel: response
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, f"channels/{channel_id}")
        return Channel(response) if is_ok else None

    async def channels_aggregate_tickets(self, channel_id: str) -> bool:
        """
        Aggregate channel tickets.
        :param: channel_id: str
        :return: bool
        """
        is_ok, _ = await self.__call_api(HTTPMethod.POST, f"channels/{channel_id}/tickets/aggregate")
        return is_ok

    async def channel_get_tickets(self, channel_id: str) -> Optional[list[Ticket]]:
        """
        Returns all channel tickets.
        :param: channel_id: str
        :return: tickets: response
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, f"channels/{channel_id}/tickets")
        return [Ticket(entry) for entry in response] if is_ok else []

    async def tickets_redeem(self):
        """
        Redeems all tickets.
        :return: bool
        """
        is_ok, _ = await self.__call_api(HTTPMethod.POST, "tickets/redeem")
        return is_ok

    async def peers(
        self,
    ) -> list[ConnectedPeer]:
        """
        Returns a list of peers.
        :return: peers: list
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "node/peers")

        if not is_ok:
            return []

        if "connected" not in response:
            logging.warning("No 'connected' field returned from the API")
            return []

        return [ConnectedPeer(peer) for peer in response["connected"]]

    async def ping(self, destination: str) -> Optional[Ping]:
        """
        Pings the given destination and returns the measure.
        :param: destination: str
        :return: response: dict
        """
        is_ok, response = await self.__call_api(HTTPMethod.POST, f"peers/{destination}/ping")
        return Ping(response) if is_ok else None

    async def addresses(self) -> Optional[Addresses]:
        """
        Returns the address of the node.
        :return: address: str | undefined
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "account/addresses")

        return Addresses(response) if is_ok else None

    async def config(self) -> Optional[Configuration]:
        """
        Returns some configurations value of the node.
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "node/configuration")

        return Configuration(response["config"]) if is_ok else None

    async def node_info(self) -> Optional[Infos]:
        """
        Gets informations about the HOPRd node.
        :return: Infos
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "node/info")
        return Infos(response) if is_ok else None

    async def ticket_price(self) -> Optional[TicketPrice]:
        """
        Gets the ticket price set by the oracle.
        :return: TicketPrice
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "network/price")
        return TicketPrice(response) if is_ok else None

    async def ticket_min_win_prob(self) -> Optional[TicketProbability]:
        """
        Gets the minimum ticket winning probability set by the oracle.
        :return: TicketProbability
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "network/probability")
        return TicketProbability(response) if is_ok else None

    async def withdraw(self, amount: str, receipient: str, currency: str):
        """
        Withdraws the given amount of token (Native or HOPR) to the given receipient.
        :param: amount: str
        :param: receipient: str
        :param: currency: str
        :return:
        """
        data = WithdrawBody(receipient, amount=f"{amount} {currency}")
        is_ok, _ = await self.__call_api(HTTPMethod.POST, "account/withdraw", data=data)
        return is_ok

    async def metrics(self):
        is_ok, response = await self.__call_api(HTTPMethod.GET, "metrics", use_api_path=False)

        return response if is_ok else None

    async def get_tickets_statistics(self) -> Optional[TicketStatistics]:
        """
        Returns the ticket statistics of the node.
        :return: statistics: dict
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, "tickets/statistics")
        return TicketStatistics(response) if is_ok else None

    async def reset_tickets_statistics(self):
        """
        Resets the ticket statistics of the node.
        :return: bool
        """
        is_ok, _ = await self.__call_api(HTTPMethod.DELETE, "tickets/statistics")
        return is_ok

    async def session_list_clients(self, protocol: Protocol = Protocol.UDP) -> list[Session]:
        """
        Lists existing Session listeners for the given IP protocol
        :param: protocol: Protocol
        :return: list[Session]
        """
        is_ok, response = await self.__call_api(HTTPMethod.GET, f"session/{protocol.name.lower()}")
        return [Session(s) for s in response] if is_ok else None

    async def session_client(
        self,
        destination: str,
        forward_path: dict,
        return_path: dict,
        protocol: Protocol,
        target: str,
        listen_on: str = "127.0.0.1:0",
        service: bool = False,
        capabilities: SessionCapabilitiesBody = SessionCapabilitiesBody(),
        sealed_target: bool = False,
        response_buffer: str = "4MiB",
    ) -> Optional[Session]:
        """
        Creates a new client session returning the given session listening host & port over TCP or UDP.
        :param destination: Address of the recipient
        :param forward_path: Forward routing options for the session.
        :param return_path: Return routing options for the session.
        :param protocol: Protocol (UDP or TCP)
        :param target: Destination for the session packets.
        :param listen_on: The host to listen on for input packets (default: "127.0.0.1:0")
        :param service: Indicates if the target is a service (default: False)
        :param capabilities: Session capabilities (default: none)
        :param sealed_target: The target parameter will be encrypted (default: False)
        :param response_buffer: The size of the response buffer to maintain at the counterparty (default: "3 MB")
        :return: Session
        """
        actual_target = (
            {"Sealed": base64.b64encode(seal_with_address(destination, bytes(target, "utf-8"), 50)).decode("ascii")}
            if sealed_target
            else {"Service": int(target)}
            if service
            else {"Plain": target}
        )

        data = CreateSessionBody(
            capabilities.as_array, destination, listen_on, forward_path, return_path, actual_target, response_buffer
        )

        is_ok, response = await self.__call_api(HTTPMethod.POST, f"session/{protocol.name.lower()}", data)

        return Session(response) if is_ok else None

    async def session_close_client(self, session: Session) -> bool:
        """
        Closes an existing Session listener for the given IP protocol, IP and port.
        :param: session: Session
        """

        is_ok, response = await self.__call_api(
            HTTPMethod.DELETE,
            f"session/{session.protocol}/{session.ip}/{session.port}",
        )

        return is_ok

    async def readyz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is ready. Return True if `readyz` returns 200 after max `timeout` seconds.
        """
        return await HoprdAPI.is_url_returning_200(f"{self.host}/readyz", timeout)

    async def healthyz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is healthy. Return True if `healthyz` returns 200 after max `timeout` seconds.
        """
        return await HoprdAPI.is_url_returning_200(f"{self.host}/healthyz", timeout)

    async def startedz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is started. Return True if `startedz` returns 200 after max `timeout` seconds.
        """
        return await HoprdAPI.is_url_returning_200(f"{self.host}/startedz", timeout)

    async def eligiblez(self, timeout: int = 20) -> bool:
        """
        Checks if the node is eligible. Return True if `eligiblez` returns 200 after max `timeout` seconds.
        """
        return await HoprdAPI.is_url_returning_200(f"{self.host}/eligiblez", timeout)

    @classmethod
    async def is_url_returning_200(cls, url, timeout):
        async def check_url():
            ready = False

            async with aiohttp.ClientSession() as s:
                while not ready:
                    try:
                        ready = (await s.get(url, timeout=0.3)).status == 200
                        await asyncio.sleep(0.5)
                    except Exception:
                        await asyncio.sleep(0.2)

                return ready

        try:
            return await asyncio.wait_for(check_url(), timeout=timeout)
        except Exception:
            return False
