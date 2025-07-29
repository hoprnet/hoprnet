import asyncio
import random

import pytest

from sdk.python.api.balance import Balance
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes
from .utils import (
    basic_send_and_receive_packets_over_single_route,
    check_all_tickets_redeemed,
    check_unredeemed_tickets_value,
    create_bidirectional_channels_for_route,
    get_ticket_price,
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

        assert statistics.unredeemed_value == Balance.zero("wxHOPR")

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_api_should_redeem_tickets_in_channel_using_redeem_all_endpoint(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 4
        relay = swarm7[random.choice(route[1:-1])]

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], (message_count + 2) * ticket_price, 2 * ticket_price
        ) as channels:
            unredeemed_tickets_before = (await relay.api.get_tickets_statistics()).unredeemed_value

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(relay, unredeemed_tickets_before + (message_count + 4) * ticket_price),
                30.0,
            )

            # Redeem using the redeem-all API
            assert await relay.api.tickets_redeem()

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

            # No remaining unredeemed tickets
            ticket_statistics = await relay.api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == Balance.zero("wxHOPR")

            # No tickets in both channels
            assert await relay.api.channel_get_tickets(channels.fwd_channels[0].id) == []
            assert await relay.api.channel_get_tickets(channels.return_channels[0].id) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_redeem_tickets_in_channel_on_api_request(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 2

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], (message_count + 2) * ticket_price, 2 * ticket_price
        ) as channels:
            relay = swarm7[random.choice(route[1:-1])]
            unredeemed_value_before = (await relay.api.get_tickets_statistics()).unredeemed_value

            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            await asyncio.wait_for(
                check_unredeemed_tickets_value(relay, unredeemed_value_before + (message_count + 4) * ticket_price),
                30.0,
            )

            # Redeem in the forward channel
            assert await relay.api.channel_redeem_tickets(channels.fwd_channels[0].id)

            # Redeem in the return channel
            assert await relay.api.channel_redeem_tickets(channels.return_channels[0].id)

            await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

            # No remaining unredeemed tickets
            ticket_statistics = await relay.api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == Balance.zero("wxHOPR")

            # No tickets in both channels
            assert await relay.api.channel_get_tickets(channels.fwd_channels[0].id) == []
            assert await relay.api.channel_get_tickets(channels.return_channels[0].id) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1, 2], barebone_nodes()))
    async def test_hoprd_should_create_redeemable_tickets_on_routing_in_n_hop(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        message_count = 5

        # Note that there are always +2 of messages in both directions due to the Session establishment and closure

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], (message_count + 2) * ticket_price, 2 * ticket_price
        ):
            # Get values of unredeemed tickets on relays
            unredeemed_values_before = []
            for relay in (swarm7[hop] for hop in route[1:-1]):
                unredeemed_values_before.append((await relay.api.get_tickets_statistics()).unredeemed_value)

            # Send packets over the route
            await basic_send_and_receive_packets_over_single_route(
                message_count,
                [swarm7[hop] for hop in route],
            )

            # Wait for tickets to be acknowledged and start redeeming them on relays
            route_len = len(route) - 2
            for i, relay in enumerate(swarm7[hop] for hop in route[1:-1]):
                # Each hop must have the unredeemed value also proportional to its position in the route
                # The economic effect of session establishment and initiation messages cancels out,
                # because they come once from each side
                await asyncio.wait_for(
                    check_unredeemed_tickets_value(
                        relay,
                        unredeemed_values_before[i]
                        + (message_count + 2) * ticket_price * (route_len - i)
                        + 2 * ticket_price * (i + 1),
                    ),
                    30.0,
                )

                assert await relay.api.tickets_redeem()

            # Wait until all tickets on relays get redeemed
            for relay in (swarm7[hop] for hop in route[1:-1]):
                await asyncio.wait_for(check_all_tickets_redeemed(relay), 120.0)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_hoprd_should_be_able_to_close_open_channels_with_unredeemed_tickets(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 2

        relay = swarm7[random.choice(route[1:-1])]

        ticket_statistics = await relay.api.get_tickets_statistics()
        neglected_value_before = ticket_statistics.neglected_value

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], (ticket_count + 2) * ticket_price, 2 * ticket_price
        ):
            await basic_send_and_receive_packets_over_single_route(ticket_count, [swarm7[hop] for hop in route])

            await asyncio.wait_for(check_unredeemed_tickets_value(relay, (ticket_count + 4) * ticket_price), 30.0)

            # NOTE: will be closed on context manager exit

        # Once channels are closed, the tickets must become neglected
        ticket_statistics = await relay.api.get_tickets_statistics()
        assert ticket_statistics.neglected_value >= neglected_value_before + (ticket_count + 4) * ticket_price
