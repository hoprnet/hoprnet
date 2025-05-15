import asyncio
import random
import socket
from contextlib import asynccontextmanager

from sdk.python.api.channelstatus import ChannelStatus
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


def gen_random_tag():
    return random.randint(APPLICATION_TAG_THRESHOLD_FOR_SESSIONS, 65530)


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


async def check_received_packets_with_pop(receiver: Node, expected_packets, tag=None, sort=True):
    received = []

    while len(received) != len(expected_packets):
        packet = await receiver.api.messages_pop(tag)
        if packet is not None:
            received.append(packet.body)
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)

    if sort:
        expected_packets.sort()
        received.sort()

    assert received == expected_packets


async def check_received_packets_with_peek(receiver: Node, expected_packets: list[str], tag=None, sort=True):
    received = []

    while len(received) != len(expected_packets):
        packets = await receiver.api.messages_peek_all(tag)

        if packets is None:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)
            continue

        received = [m.body for m in packets]

    if sort:
        expected_packets.sort()
        received.sort()

    assert received == expected_packets, f"Expected: {expected_packets}, got: {received}"


async def check_rejected_tickets_value(src: Node, value: int):
    while (await src.api.get_tickets_statistics()).rejected_value < value:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_unredeemed_tickets_value(src: Node, value: int):
    while (await src.api.get_tickets_statistics()).unredeemed_value < value:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_winning_tickets_count(src: Node, value: int):
    while (await src.api.get_tickets_statistics()).winning_count < value:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


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
    while (await src.api.get_tickets_statistics()).unredeemed_value > 0:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def send_and_receive_packets_with_pop(
    packets, src: Node, dest: Node, path: list[str], timeout: float = MULTIHOP_MESSAGE_SEND_TIMEOUT
):
    random_tag = gen_random_tag()

    for packet in packets:
        assert await src.api.send_message(dest.peer_id, packet, path, random_tag)

    await asyncio.wait_for(check_received_packets_with_pop(dest, packets, tag=random_tag, sort=True), timeout)


async def send_and_receive_packets_with_peek(
    packets, src: Node, dest: Node, path: list[str], timeout: float = MULTIHOP_MESSAGE_SEND_TIMEOUT
):
    random_tag = gen_random_tag()

    for packet in packets:
        assert await src.api.send_message(dest.peer_id, packet, path, random_tag)

    await asyncio.wait_for(check_received_packets_with_peek(dest, packets, tag=random_tag, sort=True), timeout)

    return random_tag

def find_available_port_block(min_port=9000, max_port=9980, skip=20, block_size=None):
    """
    Find a randomly selected available port on localhost within the specified range,
    checking only every nth port based on the skip parameter, and ensuring that
    a contiguous block of ports following the found port are also available.

    Args:
        min_port (int): The minimum port number to check (inclusive)
        max_port (int): The maximum port number to check (inclusive)
        skip (int): Check only every nth port (e.g. skip=2 checks every second port)
        block_size (int, optional): Number of consecutive ports that must be free.
                                   If None, defaults to the same value as skip.

    Returns:
        int: The starting port number of an available block, or None if no suitable block found
    """
    # Ensure skip is at least 0
    skip = max(0, skip)

    # If block_size is not specified, use the same value as skip
    if block_size is None:
        block_size = skip

    # Adjust max_port to ensure we can fit a block of ports at the end
    adjusted_max = max_port - block_size + 1

    # Create a list of potential starting ports in the specified range, applying the skip
    potential_starts = list(range(min_port, adjusted_max + 1, skip))

    # Randomize the port order
    random.shuffle(potential_starts)

    # Variable to store our result
    result = None

    for start_port in potential_starts:
        # Check if all ports in the block are available
        block_available = True

        # Store open sockets temporarily to prevent others from taking the ports while we're checking
        temp_sockets = []

        # Check each port in the block
        for offset in range(block_size):
            port = start_port + offset

            # Skip checking if port is beyond the max_port
            if port > max_port:
                block_available = False
                break

            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                # connect_ex returns 0 if the connection succeeds,
                # and a non-zero error code otherwise
                if s.connect_ex(("127.0.0.1", port)) == 0:
                    # Port is in use
                    block_available = False
                    break

        # If all ports in the block are available, set the result
        if block_available:
            result = start_port
            break  # Exit the loop once we find a valid block

    # Return the starting port of the available block, or None if not found
    return result
