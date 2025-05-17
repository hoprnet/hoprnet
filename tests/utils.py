from typing import Optional

import asyncio
import logging
import random
import re
import socket
from contextlib import asynccontextmanager, contextmanager

from sdk.python.api import Protocol
from sdk.python.api.channelstatus import ChannelStatus
from sdk.python.api.request_objects import SessionCapabilitiesBody
from sdk.python.localcluster.constants import TICKET_PRICE_PER_HOP
from sdk.python.localcluster.node import Node

# if os.getenv("CI", default="false") == "false" else 3
TICKET_AGGREGATION_THRESHOLD = 100
PARAMETERIZED_SAMPLE_SIZE = 1
AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP
MULTIHOP_MESSAGE_SEND_TIMEOUT = 30.0
CHECK_RETRY_INTERVAL = 0.5
RESERVED_TAG_UPPER_BOUND = 1023
APPLICATION_TAG_THRESHOLD_FOR_SESSIONS = RESERVED_TAG_UPPER_BOUND + 1


def shuffled(coll):
    random.shuffle(coll)
    return coll


def make_routes(routes_with_hops: list[int], nodes: list[Node]):
    return [shuffled(nodes)[: (hop + 2)] for hop in routes_with_hops]


@asynccontextmanager
async def create_channel(src: Node, dest: Node, funding: int, close_from_dest: bool = True, use_peer_id: bool = False):
    channel = await src.api.open_channel(dest.peer_id if use_peer_id else dest.address, str(int(funding)))
    assert channel is not None
    await asyncio.wait_for(check_channel_status(src, dest, status=ChannelStatus.Open), 10.0)
    try:
        yield channel
    finally:
        if close_from_dest:
            assert await dest.api.close_channel(channel.id)
            await asyncio.wait_for(check_channel_status(src, dest, status=ChannelStatus.Closed), 10.0)
        else:
            assert await src.api.close_channel(channel.id)
            await asyncio.wait_for(check_channel_status(src, dest, status=ChannelStatus.PendingToClose), 10.0)

            # need to wait some more time until closure time has passed and the
            # closure may be finalized
            await asyncio.sleep(15)

            assert await src.api.close_channel(channel.id)
            await asyncio.wait_for(check_channel_status(src, dest, status=ChannelStatus.Closed), 10.0)


async def get_channel(src: Node, dest: Node, include_closed=False):
    all_channels = await src.api.all_channels(include_closed=include_closed)

    channels = [
        oc for oc in all_channels.all if oc.source_address == src.address and oc.destination_address == dest.address
    ]

    return channels[0] if len(channels) > 0 else None


async def get_channel_seen_from_dst(src: Node, dest: Node, include_closed=False):
    open_channels = await dest.api.all_channels(include_closed)
    channels = [
        oc for oc in open_channels.all if oc.source_address == src.address and oc.destination_address == dest.address
    ]

    return channels[0] if len(channels) > 0 else None


async def check_channel_status(src: Node, dest: Node, status: ChannelStatus):
    include_closed = status.is_closed
    while True:
        channel = await get_channel(src, dest, include_closed)
        channel_seen_from_dst = await get_channel_seen_from_dst(src, dest, include_closed)
        if (
            channel is not None
            and channel.status == status
            and channel_seen_from_dst is not None
            and channel_seen_from_dst.status == status
        ):
            break
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_outgoing_channel_closed(src: Node, channel_id: str):
    while True:
        channel = await src.api.get_channel(channel_id)
        if channel is not None and channel.status.is_closed:
            break
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_rejected_tickets_value(src: Node, value: int):
    current = (await src.api.get_tickets_statistics()).rejected_value
    while current < value:
        logging.debug(f"Rejected tickets value: {current}, wanted min: {value}")
        await asyncio.sleep(CHECK_RETRY_INTERVAL)
        current = (await src.api.get_tickets_statistics()).rejected_value


async def check_unredeemed_tickets_value(src: Node, value: int):
    current = (await src.api.get_tickets_statistics()).unredeemed_value
    while current < value:
        logging.debug(f"Unredeemed tickets value: {current}, wanted min: {value}")
        await asyncio.sleep(CHECK_RETRY_INTERVAL)
        current = (await src.api.get_tickets_statistics()).unredeemed_value


async def check_winning_tickets_count(src: Node, value: int):
    current = (await src.api.get_tickets_statistics()).winning_count
    while current < value:
        logging.debug(f"Winning tickets count: {current}, wanted min: {value}")
        await asyncio.sleep(CHECK_RETRY_INTERVAL)
        current = (await src.api.get_tickets_statistics()).winning_count


async def check_safe_balance(src: Node, value: int):
    while f"{(await src.api.balances()).safe_hopr:.0f}" >= f"{value:.0f}":
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_native_balance_below(src: Node, value: int):
    while (await src.api.balances()).native >= value:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_min_incoming_win_prob_eq(src: Node, value: float):
    while round((await src.api.ticket_min_win_prob()).value, 5) != value:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_all_tickets_redeemed(src: Node):
    current = (await src.api.get_tickets_statistics()).unredeemed_value
    while current > 0:
        logging.debug(f"Unredeemed tickets value: {current}, wanted max: 0")
        await asyncio.sleep(CHECK_RETRY_INTERVAL)
        current = (await src.api.get_tickets_statistics()).unredeemed_value


async def get_ticket_price(src: Node):
    ticket_price = await src.api.ticket_price()
    assert ticket_price is not None
    logging.debug(f"Ticket price: {ticket_price}")
    return ticket_price.value


from contextlib import AsyncExitStack


class RouteBidirectionalChannels:
    def __init__(self, route: list[Node], funding_fwd: int, funding_return: int):
        self.channels = []
        self.route = route
        self.funding_fwd = funding_fwd
        self.funding_return = funding_return
        self.exit_stack = AsyncExitStack()  # We'll use this for channel management

    async def __aenter__(self):
        # Enter AsyncExitStack to manage channels
        await self.exit_stack.__aenter__()
        channels_to = [
            self.exit_stack.enter_async_context(
                create_channel(
                    self.route[i],
                    self.route[i + 1],
                    funding=self.funding_fwd,
                )
            )
            for i in range(len(self.route) - 1)
        ]
        channels_back = [
            self.exit_stack.enter_async_context(
                create_channel(
                    self.route[i],
                    self.route[i - 1],
                    funding=self.funding_return,
                )
            )
            for i in reversed(range(1, len(self.route)))
        ]

        self.channels = await asyncio.gather(*(channels_to + channels_back))
        return self  # Return the context so it can be used in the block

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        # Delegate exit handling to the exit stack
        await self.exit_stack.__aexit__(exc_type, exc_val, exc_tb)

    @property
    def fwd_channels(self):
        return self.channels[: len(self.route) - 1]

    @property
    def return_channels(self):
        return self.channels[len(self.route) - 1 :]


def create_bidirectional_channels_for_route(route: list[Node], funding_fwd: int, funding_return: int):
    return RouteBidirectionalChannels(route, funding_fwd, funding_return)


class HoprSession:
    def __init__(
        self,
        proto: Protocol,
        src: Node,
        dest: Node,
        fwd_path: dict,
        return_path: dict,
        capabilities: SessionCapabilitiesBody = SessionCapabilitiesBody(),
        use_response_buffer: Optional[str] = "1 MiB",
        dummy_server_listen_port: Optional[int] = 0,
        loopback: bool = False,
    ):
        self._src = src
        self._dest = dest
        self._proto = proto
        self._fwd_path = fwd_path
        self._return_path = return_path
        self._capabilities = capabilities
        self._session = None
        self._dummy_server_sock = None
        self._target_port = 0
        self._use_response_buffer = use_response_buffer
        self._dummy_server_binding_port = dummy_server_listen_port
        self._loopback = loopback

    async def __aenter__(self):
        if self._dummy_server_binding_port is not None and self._loopback is False:
            if self._proto is Protocol.TCP:
                self._dummy_server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            else:
                self._dummy_server_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

            self._dummy_server_sock.bind(("127.0.0.1", self._dummy_server_binding_port))
            self._target_port = self._dummy_server_sock.getsockname()[1]
            logging.debug(
                f"Bound listening socket 127.0.0.1:{self._target_port} on {self._proto.name} for future Session"
            )

            if self._proto is Protocol.TCP:
                self._dummy_server_sock.listen()

        target = f"127.0.0.1:{self._target_port}"
        if self._loopback is True:
            target = "0"

        resp_buffer = "0 MiB"
        if self._use_response_buffer is not None:
            resp_buffer = self._use_response_buffer

        self._session = await self._src.api.session_client(
            self._dest.peer_id,
            forward_path=self._fwd_path,
            return_path=self._return_path,
            protocol=self._proto,
            target=target,
            capabilities=self._capabilities,
            response_buffer=resp_buffer,
            service=self._loopback,
        )
        if self._session is None:
            raise Exception(f"Failed to open session {self._src.peer_id} -> {self._dest.peer_id} on {self._proto.name}")

        logging.debug(
            f"Session opened {self._src.peer_id}:{self._session.port} -> {self._dest.peer_id}:{self._target_port} on {self._proto.name}"
        )
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session is not None and await self._src.api.session_close_client(self._session) is True:
            logging.debug(
                f"Session closed {self._src.peer_id}:{self._session.port} -> {self._dest.peer_id}:{self._target_port} on {self._proto.name}"
            )
            self._session = None
            self._target_port = 0
            if self._dummy_server_sock is not None:
                self._dummy_server_sock.close()
        else:
            logging.error("Failed to close session")

    @contextmanager
    def client_socket(self):
        if self._session is None:
            raise Exception("Session is not open")

        if self._proto is Protocol.TCP:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.connect(("127.0.0.1", self._session.port))
        else:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

        try:
            logging.debug(f"Connected session client to 127.0.0.1:{self._session.port} on {self._proto.name}")
            yield s
        finally:
            s.close()

    @property
    def mtu(self):
        if self._session is None:
            raise Exception("Session is not open")
        return self._session.mtu

    @property
    def listen_port(self):
        if self._session is None:
            raise Exception("Session is not open")
        return self._session.port

    @property
    def target_port(self):
        if self._session is None:
            raise Exception("Session is not open")
        return self._target_port

    @contextmanager
    def server_socket(self):
        if self._session is None:
            raise Exception("Session is not open")
        if self._dummy_server_sock is None:
            raise Exception("Server socket not configured")

        try:
            yield self._dummy_server_sock
        finally:
            self._dummy_server_sock.close()
            self._dummy_server_sock = None


async def basic_send_and_receive_packets(
    msg_count: int,
    src: Node,
    dest: Node,
    fwd_path: dict,
    return_path: dict,
):
    async with HoprSession(
        Protocol.UDP,
        src,
        dest,
        fwd_path,
        return_path,
        SessionCapabilitiesBody(no_delay=True, segmentation=True),
        use_response_buffer=None,
    ) as session:
        addr = ("127.0.0.1", session.listen_port)
        msg_len = int(session.mtu / 2)  # Allow space for SURBs, since no response buffer is used

        expected = [f"#{i}".ljust(msg_len) for i in range(msg_count)]
        actual = []
        total_sent = 0

        with session.client_socket() as s:
            s.settimeout(5)
            logging.debug(f"Sending {msg_count} UDP messages to 127.0.0.1:{session.listen_port}")
            for message in expected:
                total_sent = total_sent + s.sendto(message.encode(), addr)
                # UDP has no flow-control, so we must insert an artificial gap
                await asyncio.sleep(0.01)

        logging.debug(f"Sent {total_sent} bytes")

        logging.debug(f"Receiving {msg_count} UDP messages at 127.0.0.1:{session.target_port}")
        with session.server_socket() as s:
            s.settimeout(5)
            while total_sent > 0:
                chunk, _ = s.recvfrom(min(msg_len, total_sent))
                logging.debug(f"Received {len(chunk)} bytes")
                total_sent = total_sent - len(chunk)

                # Adapt for situations when data arrive completely unordered (also within the buffer)
                actual.extend([m for m in re.split(r"\s+", chunk.decode().strip()) if len(m) > 0])

        logging.debug(f"All bytes received")
        expected = [msg.strip() for msg in expected]

        actual.sort()
        expected.sort()

        assert "".join(expected) == "".join(actual)


async def basic_send_and_receive_packets_over_single_route(msg_count: int, route: list[Node]):
    await basic_send_and_receive_packets(
        msg_count,
        src=route[0],
        dest=route[-1],
        fwd_path={"IntermediatePath": [n.peer_id for n in route[1:-1]]},
        return_path={"IntermediatePath": [n.peer_id for n in route[-1:1]]},
    )
