import logging

import asyncio

import pytest
import random

from sdk.python.api.hopr import HoprdAPI
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes
from .utils import (
    TICKET_AGGREGATION_THRESHOLD,
    check_all_tickets_redeemed,
    check_unredeemed_tickets_value,
    get_ticket_price,
    create_bidirectional_channels_for_route,
    basic_send_and_receive_packets_over_single_route,
    make_routes,
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
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_api_should_redeem_tickets_in_channel_using_redeem_endpoint(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], message_count * ticket_price, ticket_price
        ) as channels:
            relay = swarm7[random.choice(route[1:-1])]
            unredeemed_tickets_before = (await relay.api.get_tickets_statistics()).unredeemed_value

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(relay, unredeemed_tickets_before + (message_count + 2) * ticket_price),
                30.0,
            )

            async def channel_redeem_tickets(api: HoprdAPI, channel: str):
                while True:
                    if await api.channel_redeem_tickets(channel):
                        break
                    else:
                        await asyncio.sleep(0.5)

            logging.debug(f"redeeming all tickets in channel {channels.fwd_channels[0].id}")
            await asyncio.wait_for(channel_redeem_tickets(relay.api, channels.fwd_channels[0].id), 20.0)

            logging.debug(f"redeeming all tickets in channel {channels.return_channels[0].id}")
            await asyncio.wait_for(channel_redeem_tickets(relay.api, channels.return_channels[0].id), 20.0)

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

            assert await relay.api.channel_get_tickets(channels.fwd_channels[0].id) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_create_redeemable_tickets_on_routing_in_1_hop(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        # send 90% of messages before ticket aggregation would kick in
        message_count = int(TICKET_AGGREGATION_THRESHOLD / 10 * 9)

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], message_count * ticket_price, ticket_price
        ) as channels:
            relay = swarm7[random.choice(route[1:-1])]
            statistics_before = await relay.api.get_tickets_statistics()

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(
                    relay, statistics_before.unredeemed_value + message_count * ticket_price
                ),
                30.0,
            )

            # ensure ticket stats are updated after messages are sent
            statistics_after = await relay.api.get_tickets_statistics()

            assert statistics_after.redeemed_value == statistics_before.redeemed_value

            assert await relay.api.channel_redeem_tickets(channels.fwd_channels[0].id)
            assert await relay.api.channel_redeem_tickets(channels.return_channels[0].id)

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

            # ensure ticket stats are updated after redemption
            statistics_after_redemption = await relay.api.get_tickets_statistics()
            assert (statistics_after_redemption.redeemed_value - statistics_after.redeemed_value) == (
                message_count * ticket_price
            )
            assert statistics_after_redemption.unredeemed_value == 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_redeem_tickets_in_channel_on_api_request(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], message_count * ticket_price, ticket_price
        ) as channels:
            relay = swarm7[random.choice(route[1:-1])]
            unredeemed_value_before = (await relay.api.get_tickets_statistics()).unredeemed_value

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(relay, unredeemed_value_before + message_count * ticket_price),
                30.0,
            )

            # Ticket aggregation does not change unredeemed value
            # await asyncio.wait_for(relay.api.channels_aggregate_tickets(channels.fwd_channels[0].id), 20.0)
            # await asyncio.wait_for(relay.api.channels_aggregate_tickets(channels.return_channels[0].id), 20.0)
            # ticket_statistics = await relay.api.get_tickets_statistics()
            # assert ticket_statistics.unredeemed_value == unredeemed_value_after

            assert await relay.api.channel_redeem_tickets(channels.fwd_channels[0].id)
            assert await relay.api.channel_redeem_tickets(channels.return_channels[0].id)

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

            # No remaining unredeemed tickets
            ticket_statistics = await relay.api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_create_redeemable_tickets_on_routing_in_general_n_hop(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 5

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], message_count * ticket_price, ticket_price
        ):
            relay = swarm7[random.choice(route[1:-1])]
            unredeemed_value_before = (await relay.api.get_tickets_statistics()).unredeemed_value

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(relay, unredeemed_value_before + message_count * ticket_price),
                30.0,
            )

            assert await relay.api.tickets_redeem()

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_be_able_to_close_open_channels_with_unredeemed_tickets(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], ticket_count * ticket_price, ticket_price
        ):
            relay = swarm7[random.choice(route[1:-1])]
            await basic_send_and_receive_packets_over_single_route(ticket_count, [swarm7[hop] for hop in route])

            await asyncio.wait_for(check_unredeemed_tickets_value(relay, ticket_count * ticket_price), 30.0)

            # NOTE: will be closed on context manager exit
