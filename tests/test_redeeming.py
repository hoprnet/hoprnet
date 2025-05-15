import asyncio

import pytest

from sdk.python.api.hopr import HoprdAPI
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes
from .utils import (
    TICKET_AGGREGATION_THRESHOLD,
    PARAMETERIZED_SAMPLE_SIZE,
    check_all_tickets_redeemed,
    check_unredeemed_tickets_value,
    shuffled,
    get_ticket_price,
    create_bidirectional_channels_for_route,
    session_send_and_receive_packets_over_single_route,
)


@pytest.mark.usefixtures("swarm7_reset")
class TestRedeemingWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", barebone_nodes())
    async def test_hoprd_should_not_have_unredeemed_tickets_without_sending_messages(
        self, peer: str, swarm7: dict[str, Node]
    ):
        statistics = await swarm7[peer].api.get_tickets_statistics()

        assert statistics.unredeemed_value == 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,mid,dest", [tuple(shuffled(barebone_nodes())[:3]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_hoprd_api_should_redeem_tickets_in_channel_using_redeem_endpoint(
        self, src: str, mid: str, dest: str, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[src])
        message_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[mid], swarm7[dest]], message_count * ticket_price, ticket_price
        ) as channels:
            unredeemed_tickets_before = await swarm7[mid].api.get_tickets_statistics().unredeemed_tickets

            await session_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[src], swarm7[mid], swarm7[dest]],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(swarm7[mid], unredeemed_tickets_before + message_count * ticket_price),
                30.0,
            )

            async def channel_redeem_tickets(api: HoprdAPI, channel: str):
                while True:
                    if await api.channel_redeem_tickets(channel):
                        break
                    else:
                        await asyncio.sleep(0.5)

            await asyncio.wait_for(channel_redeem_tickets(swarm7[mid].api, channels.fwd_channels[0].id), 20.0)

            await asyncio.wait_for(check_all_tickets_redeemed(swarm7[mid]), 120.0)

            assert await swarm7[mid].api.channel_get_tickets(channels.fwd_channels[0].id) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_hoprd_should_create_redeemable_tickets_on_routing_in_1_hop_to_self_scenario(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[src])
        # send 90% of messages before ticket aggregation would kick in
        message_count = int(TICKET_AGGREGATION_THRESHOLD / 10 * 9)
        assert message_count < TICKET_AGGREGATION_THRESHOLD

        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[dest]], message_count * ticket_price, ticket_price
        ) as channels:
            statistics_before = await swarm7[dest].api.get_tickets_statistics()

            await session_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[src], swarm7[dest]],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(
                    swarm7[dest], statistics_before.unredeemed_value + message_count * ticket_price
                ),
                30.0,
            )

            # ensure ticket stats are updated after messages are sent
            statistics_after = await swarm7[dest].api.get_tickets_statistics()

            assert statistics_after.redeemed_value == statistics_before.redeemed_value

            assert await swarm7[dest].api.channel_redeem_tickets(channels.fwd_channels[0].id)

            await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

            # ensure ticket stats are updated after redemption
            statistics_after_redemption = await swarm7[dest].api.get_tickets_statistics()
            assert (statistics_after_redemption.redeemed_value - statistics_after.redeemed_value) == (
                message_count * ticket_price
            )
            assert statistics_after_redemption.unredeemed_value == 0

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="ticket aggregation is not implemented as a session protocol yet")
    @pytest.mark.parametrize(
        "src,mid,dest", [tuple(shuffled(barebone_nodes())[:3]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_hoprd_should_aggregate_and_redeem_tickets_in_channel_on_api_request(
        self, src: str, mid: str, dest: str, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[src])
        message_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[mid], swarm7[dest]], message_count * ticket_price, ticket_price
        ) as channels:
            unredeemed_value_before = (await swarm7[mid].api.get_tickets_statistics()).unredeemed_value
            await session_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[src], swarm7[mid], swarm7[dest]],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(swarm7[mid], unredeemed_value_before + message_count * ticket_price),
                30.0,
            )

            unredeemed_value_after = (await swarm7[mid].api.get_tickets_statistics()).unredeemed_value
            first_channel_id = channels.fwd_channels[0].id

            await asyncio.wait_for(swarm7[mid].api.channels_aggregate_tickets(first_channel_id), 20.0)

            # Ticket aggregation does not change unredeemed value
            ticket_statistics = await swarm7[mid].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == unredeemed_value_after

            assert await swarm7[mid].api.channel_redeem_tickets(first_channel_id)

            await asyncio.wait_for(check_all_tickets_redeemed(swarm7[mid]), 120.0)

            # No remaining unredeemed tickets
            ticket_statistics = await swarm7[mid].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
        # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    )
    async def test_hoprd_should_create_redeemable_tickets_on_routing_in_general_n_hop(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 5

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], message_count * ticket_price, ticket_price
        ):
            unredeemed_value_before = (await swarm7[route[1]].api.get_tickets_statistics()).unredeemed_value

            await session_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(
                    swarm7[route[1]], unredeemed_value_before + message_count * ticket_price
                ),
                30.0,
            )

            assert await swarm7[route[1]].api.tickets_redeem()

            await asyncio.wait_for(check_all_tickets_redeemed(swarm7[route[1]]), 120.0)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
    async def test_hoprd_should_be_able_to_close_open_channels_with_unredeemed_tickets(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], ticket_count * ticket_price, ticket_price
        ):
            await session_send_and_receive_packets_over_single_route(ticket_count, [swarm7[hop] for hop in route])

            await asyncio.wait_for(check_unredeemed_tickets_value(swarm7[route[1]], ticket_count * ticket_price), 30.0)

            # NOTE: will be closed on context manager exit
