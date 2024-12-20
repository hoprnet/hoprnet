import asyncio
from contextlib import AsyncExitStack

import pytest

from sdk.python.api.hopr import HoprdAPI
from sdk.python.localcluster.constants import (
    TICKET_AGGREGATION_THRESHOLD,
    TICKET_PRICE_PER_HOP,
)
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes
from .utils import (
    PARAMETERIZED_SAMPLE_SIZE,
    check_all_tickets_redeemed,
    check_unredeemed_tickets_value,
    create_channel,
    send_and_receive_packets_with_pop,
    shuffled,
)

# used by nodes to get unique port assignments
PORT_BASE = 19000


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", barebone_nodes())
async def test_hoprd_should_not_have_unredeemed_tickets_without_sending_messages(peer: str, swarm7: dict[str, Node]):
    statistics = await swarm7[peer].api.get_tickets_statistics()

    assert statistics.unredeemed_value == 0


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_api_should_redeem_tickets_in_channel_using_redeem_endpoint(
    src: str, dest: str, swarm7: dict[str, Node]
):
    message_count = 2

    async with create_channel(
        swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP, close_from_dest=False
    ) as channel:
        packets = [f"Channel redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]

        await send_and_receive_packets_with_pop(packets, src=swarm7[src], dest=swarm7[src], path=[swarm7[dest].peer_id])

        await asyncio.wait_for(check_unredeemed_tickets_value(swarm7[dest], message_count * TICKET_PRICE_PER_HOP), 30.0)

        async def channel_redeem_tickets(api: HoprdAPI, channel: str):
            while True:
                if await api.channel_redeem_tickets(channel):
                    break
                else:
                    await asyncio.sleep(0.5)

        await asyncio.wait_for(channel_redeem_tickets(swarm7[dest].api, channel.id), 20.0)

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

        assert await swarm7[dest].api.channel_get_tickets(channel) == []


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_1_hop_to_self_scenario(
    src: str, dest: str, swarm7: dict[str, Node]
):
    # send 90% of messages before ticket aggregation would kick in
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10 * 9)

    async with create_channel(
        swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP, close_from_dest=False
    ) as channel:
        # ensure ticket stats are what we expect before starting
        statistics_before = await swarm7[dest].api.get_tickets_statistics()
        assert statistics_before.unredeemed_value == 0

        packets = [
            f"1 hop message to self: {src} - {dest} - {src} #{i:08d} of #{message_count:08d}"
            for i in range(message_count)
        ]
        await send_and_receive_packets_with_pop(
            packets, src=swarm7[src], dest=swarm7[src], path=[swarm7[dest].peer_id], timeout=60.0
        )

        await asyncio.wait_for(check_unredeemed_tickets_value(swarm7[dest], message_count * TICKET_PRICE_PER_HOP), 30.0)

        # ensure ticket stats are updated after messages are sent
        statistics_after = await swarm7[dest].api.get_tickets_statistics()

        unredeemed_value = statistics_after.unredeemed_value - statistics_before.unredeemed_value

        assert statistics_after.redeemed_value == statistics_before.redeemed_value
        assert unredeemed_value == (len(packets) * TICKET_PRICE_PER_HOP)

        assert await swarm7[dest].api.channel_redeem_tickets(channel.id)

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

        # ensure ticket stats are updated after redemption
        statistics_after_redemption = await swarm7[dest].api.get_tickets_statistics()
        assert (statistics_after_redemption.redeemed_value - statistics_after.redeemed_value) == (
            len(packets) * TICKET_PRICE_PER_HOP
        )
        assert statistics_after_redemption.unredeemed_value == 0


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_should_aggregate_and_redeem_tickets_in_channel_on_api_request(
    src: str, dest: str, swarm7: dict[str, Node]
):
    message_count = 2

    async with create_channel(swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP) as channel:
        packets = [f"Channel agg and redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]
        await send_and_receive_packets_with_pop(packets, src=swarm7[src], dest=swarm7[src], path=[swarm7[dest].peer_id])

        await asyncio.wait_for(check_unredeemed_tickets_value(swarm7[dest], message_count * TICKET_PRICE_PER_HOP), 30.0)

        ticket_statistics = await swarm7[dest].api.get_tickets_statistics()
        assert ticket_statistics.unredeemed_value == 2 * TICKET_PRICE_PER_HOP

        await asyncio.wait_for(swarm7[dest].api.channels_aggregate_tickets(channel.id), 20.0)

        ticket_statistics = await swarm7[dest].api.get_tickets_statistics()
        assert ticket_statistics.unredeemed_value == 2 * TICKET_PRICE_PER_HOP

        assert await swarm7[dest].api.channel_redeem_tickets(channel.id)

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

        ticket_statistics = await swarm7[dest].api.get_tickets_statistics()
        assert ticket_statistics.unredeemed_value == 0


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_general_n_hop(route, swarm7: dict[str, Node]):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=message_count * TICKET_PRICE_PER_HOP)
                )
                for i in range(len(route) - 1)
            ]
        )

        packets = [f"General n-hop over {route} message #{i:08d}" for i in range(message_count)]
        await send_and_receive_packets_with_pop(
            packets,
            src=swarm7[route[0]],
            dest=swarm7[route[-1]],
            path=[swarm7[x].peer_id for x in route[1:-1]],
        )

        await asyncio.wait_for(
            check_unredeemed_tickets_value(swarm7[route[1]], message_count * TICKET_PRICE_PER_HOP), 30.0
        )

        # wait for aggregation to finish before redeeming
        await asyncio.sleep(10)
        assert await swarm7[route[1]].api.tickets_redeem()

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[route[1]]), 120.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("route", [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_should_be_able_to_close_open_channels_with_unredeemed_tickets(route, swarm7: dict[str, Node]):
    ticket_count = 2

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=ticket_count * TICKET_PRICE_PER_HOP)
                )
                for i in range(len(route) - 1)
            ]
        )

        packets = [f"Channel unredeemed check: #{i:08d}" for i in range(ticket_count)]
        await send_and_receive_packets_with_pop(
            packets, src=swarm7[route[0]], dest=swarm7[route[-1]], path=[swarm7[route[1]].peer_id]
        )

        await asyncio.wait_for(
            check_unredeemed_tickets_value(swarm7[route[1]], ticket_count * TICKET_PRICE_PER_HOP), 30.0
        )

        # NOTE: will be closed on context manager exit
