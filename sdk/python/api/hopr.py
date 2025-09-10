import base64
import logging
import random
from typing import Optional

import base58
import nacl.bindings
import nacl.public
import nacl.signing  # Ensure nacl.signing is imported correctly
import nacl.utils
from api_lib import ApiLib
from api_lib.method import Method
from nacl.public import SealedBox  # Import SealedBox explicitly

from .balance import Balance
from .protocol import Protocol
from .request_objects import (
    CreateSessionBody,
    FundChannelBody,
    GetChannelsBody,
    GetPeersBody,
    OpenChannelBody,
    SessionCapabilitiesBody,
    SetSessionConfigBody,
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
    Metrics,
    OpenedChannel,
    Ping,
    Session,
    SessionConfig,
    Ticket,
    TicketPrice,
    TicketProbability,
    TicketStatistics,
)

MESSAGE_TAG = 0x1245


def getlogger():
    logging.getLogger("api-lib").setLevel(logging.CRITICAL)
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


class HoprdAPI(ApiLib):
    """
    HOPRd API helper to handle exceptions and logging.
    """

    async def api_version(self) -> Optional[str]:
        """
        Returns the API version of the HOPRd node.
        :return: version: str | undefined
        """

        openapi_spec = await self.try_req(Method.GET, "/api-docs/openapi.json", dict, use_api_prefix=False)
        version = openapi_spec.get("info", {}).get("version", None) if isinstance(openapi_spec, dict) else None

        return version

    async def balances(self) -> Optional[Balances]:
        """
        Returns the balance of the node.
        :return: balances: Balances | undefined
        """
        return await self.try_req(Method.GET, "/account/balances", Balances)

    async def open_channel(self, destination: str, amount: Balance) -> Optional[OpenedChannel]:
        """
        Opens a channel with the given peer_address and amount.
        :param: peer_address: str
        :param: amount: Balance
        :return: channel id: str | undefined
        """
        data = OpenChannelBody(amount.as_str, destination)
        return await self.try_req(Method.POST, "/channels", OpenedChannel, data=data)

    async def fund_channel(self, channel_id: str, amount: Balance) -> bool:
        """
        Funds a given channel.
        :param: channel_id: str
        :param: amount: float
        :return: bool
        """
        data = FundChannelBody(amount.as_str)
        return await self.try_req(Method.POST, f"/channels/{channel_id}/fund", data=data, return_state=True)

    async def close_channel(self, channel_id: str) -> bool:
        """
        Closes a given channel.
        :param: channel_id: str
        :return: bool
        """
        return await self.try_req(Method.DELETE, f"/channels/{channel_id}", return_state=True)

    async def channel_redeem_tickets(self, channel_id: str) -> bool:
        """
        Redeems tickets in a specific channel.
        :param: channel_id: str
        :return: bool
        """
        return await self.try_req(Method.POST, f"/channels/{channel_id}/tickets/redeem", return_state=True)

    async def all_channels(self, include_closed: bool) -> Optional[Channels]:
        """
        Returns all channels.
        :return: channels: list
        """
        params = GetChannelsBody(True, include_closed)
        response = await self.try_req(Method.GET, f"/channels?{params.as_header_string}", dict)
        return Channels(response, "all") if response else response

    async def incoming_channels(self, include_closed: bool = False):
        """
        Returns all incoming channels.
        :return: channels: list
        """
        params = GetChannelsBody(True, include_closed)
        response = await self.try_req(Method.GET, f"/channels?{params.as_header_string}", dict)
        return Channels(response, "incoming") if response else response

    async def outgoing_channels(self, include_closed: bool = False):
        """
        Returns all outgoing channels.
        :return: channels: list
        """
        params = GetChannelsBody(True, include_closed)
        response = await self.try_req(Method.GET, f"/channels?{params.as_header_string}", dict)

        return Channels(response, "outgoing") if response else response

    async def get_channel(self, channel_id: str) -> Optional[Channel]:
        """
        Returns the channel object.
        :param: channel_id: str
        :return: channel: response
        """
        return await self.try_req(Method.GET, f"/channels/{channel_id}", Channel)

    async def channels_aggregate_tickets(self, channel_id: str) -> bool:
        """
        Aggregate channel tickets.
        :param: channel_id: str
        :return: bool
        """
        return await self.try_req(Method.POST, f"/channels/{channel_id}/tickets/aggregate", return_state=True)

    async def channel_get_tickets(self, channel_id: str) -> Optional[list[Ticket]]:
        """
        Returns all channel tickets.
        :param: channel_id: str
        :return: tickets: response
        """
        return await self.try_req(Method.GET, f"/channels/{channel_id}/tickets", list[Ticket])

    async def tickets_redeem(self):
        """
        Redeems all tickets.
        :return: bool
        """
        return await self.try_req(Method.POST, "/tickets/redeem", return_state=True)

    async def peers(
        self,
        quality: float = 0.1,
        status: str = "connected",
    ) -> list[ConnectedPeer]:
        """
        Returns a list of peers.
        :return: peers: list
        """
        params = GetPeersBody(quality)

        if r := await self.try_req(Method.GET, f"/node/peers?{params.as_header_string}"):
            return [ConnectedPeer(peer) for peer in r.get(status, [])]
        else:
            return []

    async def ping(self, destination: str) -> Optional[Ping]:
        """
        Pings the given destination and returns the measure.
        :param: destination: str
        :return: response: dict
        """
        return await self.try_req(Method.POST, f"/peers/{destination}/ping", Ping)

    async def addresses(self) -> Optional[Addresses]:
        """
        Returns the address of the node.
        :return: address: str | undefined
        """
        return await self.try_req(Method.GET, "/account/addresses", Addresses)

    async def config(self) -> Optional[Configuration]:
        """
        Returns some configurations value of the node.
        """
        return await self.try_req(Method.GET, "/node/configuration", Configuration)

    async def node_info(self) -> Optional[Infos]:
        """
        Gets informations about the HOPRd node.
        :return: Infos
        """
        return await self.try_req(Method.GET, "/node/info", Infos)

    async def ticket_price(self) -> Optional[TicketPrice]:
        """
        Gets the ticket price set by the oracle.
        :return: TicketPrice
        """
        return await self.try_req(Method.GET, "/network/price", TicketPrice)

    async def ticket_min_win_prob(self) -> Optional[TicketProbability]:
        """
        Gets the minimum ticket winning probability set by the oracle.
        :return: TicketProbability
        """
        return await self.try_req(Method.GET, "/network/probability", TicketProbability)

    async def withdraw(self, amount: Balance, recipient: str):
        """
        Withdraws the given amount of token (Native or HOPR) to the given recipient.
        :param: amount: str
        :param: recipient: str
        :return:
        """
        data = WithdrawBody(recipient, amount=amount.as_str)
        return await self.try_req(Method.POST, "/account/withdraw", data=data, return_state=True)

    async def metrics(self) -> Optional[Metrics]:
        return await self.try_req(Method.GET, "/metrics", Metrics, use_api_prefix=False)

    async def get_tickets_statistics(self) -> Optional[TicketStatistics]:
        """
        Returns the ticket statistics of the node.
        :return: statistics: dict
        """
        return await self.try_req(Method.GET, "/tickets/statistics", TicketStatistics)

    async def reset_tickets_statistics(self):
        """
        Resets the ticket statistics of the node.
        :return: bool
        """
        return await self.try_req(Method.DELETE, "/tickets/statistics", return_state=True)

    async def session_list_clients(self, protocol: Protocol = Protocol.UDP) -> Optional[list[Session]]:
        """
        Lists existing Session listeners for the given IP protocol
        :param: protocol: Protocol
        :return: list[Session]
        """
        return await self.try_req(Method.GET, f"/session/{protocol.name.lower()}", list[Session])

    async def session_get_config(self, session_id: str) -> Optional[SessionConfig]:
        """
        Gets the configurable parameters of a Session.
        :param: session_id: String
        :return: SessionConfig
        """
        return await self.try_req(Method.GET, f"/session/config/{session_id}", SessionConfig)

    async def session_set_config(self, session_id: str, cfg: SessionConfig):
        """
        Sets the configurable parameters of a Session.
        :param: session_id: String
        :param: cfg: SessionConfig
        :return: SessionConfig
        """
        data = SetSessionConfigBody(cfg.response_buffer, cfg.max_surb_upstream)
        return await self.try_req(Method.POST, f"/session/config/{session_id}", data=data)

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

        return await self.try_req(Method.POST, f"/session/{protocol.name.lower()}", Session, data=data)

    async def session_close_client(self, session: Session) -> bool:
        """
        Closes an existing Session listener for the given IP protocol, IP address and port.
        :param: session: Session
        """
        path = f"/session/{session.protocol}/{session.ip}/{session.port}"
        return await self.try_req(Method.DELETE, path, return_state=True)

    async def readyz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is ready. Return True if `readyz` returns 200 after max `timeout` seconds.
        """
        return await self.timeout_check_success("/readyz", timeout)

    async def healthyz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is healthy. Return True if `healthyz` returns 200 after max `timeout` seconds.
        """
        return await self.timeout_check_success("/healthyz", timeout)

    async def startedz(self, timeout: int = 20) -> bool:
        """
        Checks if the node is started. Return True if `startedz` returns 200 after max `timeout` seconds.
        """
        return await self.timeout_check_success("/startedz", timeout)

    async def eligiblez(self, timeout: int = 20) -> bool:
        """
        Checks if the node is eligible. Return True if `eligiblez` returns 200 after max `timeout` seconds.
        """
        return await self.timeout_check_success("/eligiblez", timeout)
