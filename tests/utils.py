import asyncio
import random
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
