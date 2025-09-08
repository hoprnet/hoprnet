import asyncio
import logging
import random
from decimal import Decimal

import pytest

from sdk.python.api import Protocol
from sdk.python.api.balance import Balance
from sdk.python.localcluster.constants import ANVIL_CONFIG_FILE, CONTRACTS_DIR, NETWORK
from sdk.python.localcluster.node import Node
from sdk.python.localcluster.utils import load_private_key

from .conftest import barebone_nodes, nodes_with_lower_outgoing_win_prob, run_hopli_cmd
from .utils import (
    PARAMETERIZED_SAMPLE_SIZE,
    HoprSession,
    basic_send_and_receive_packets_over_single_route,
    check_min_incoming_win_prob_eq,
    check_rejected_tickets_value,
    check_unredeemed_tickets_value,
    check_unredeemed_tickets_value_max,
    check_winning_tickets_count,
    create_bidirectional_channels_for_route,
    get_ticket_price,
)


def generate_anvil_endpoint(base_port: int) -> str:
    return f"http://127.0.0.1:{base_port}"


def set_minimum_winning_probability_in_network(private_key: str, win_prob: Decimal, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

    custom_env = {"PRIVATE_KEY": private_key}
    cmd = [
        "hopli",
        "win-prob",
        "set",
        "--network",
        NETWORK,
        "--contracts-root",
        CONTRACTS_DIR,
        "--winning-probability",
        str(win_prob),
        "--provider-url",
        anvil_endpoint,
    ]
    run_hopli_cmd(cmd, custom_env)


@pytest.mark.usefixtures("swarm7_reset")
class TestWinProbWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hoprd_check_min_incoming_ticket_win_prob_is_default(
        self, peer, swarm7: dict[str, Node], base_port: int
    ):
        win_prob = await swarm7[peer].api.ticket_min_win_prob()

        assert win_prob is not None
        assert 0.0 <= win_prob.value <= 1.0

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        new_win_prob = win_prob.value / 2
        set_minimum_winning_probability_in_network(private_key, new_win_prob, base_port)

        try:
            await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[peer], new_win_prob), timeout=20.0)
        finally:
            # Restore the winning probability regardless of the outcome
            set_minimum_winning_probability_in_network(private_key, win_prob.value, base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [
            [
                *random.sample(nodes_with_lower_outgoing_win_prob(), 1),
                *random.sample(barebone_nodes(), 2),
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_should_relay_packets_with_lower_win_prob_then_redeem_them(
        self, route, swarm7: dict[str, Node], base_port: int
    ):
        assert len(route) == len(set(route))  # checks that all elements of route are unique. Should never fail

        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 100
        win_prob = Decimal("0.1")
        win_ticket_tolerance = Decimal("0.3")
        relay = route[1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)
        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        for node in route:
            await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[node], win_prob), 10.0)

        # Situation:
        #
        # Session: A -> B -> C
        # A sends 10 messages to C
        # A has win prob 0.1 on tickets, everyone else has 1.0
        #
        # So B (relay) should get:
        #  - 102 tickets (1 session establish + 100 messages + 1 session closure) from A with win prob 0.1.
        #  - 2 tickets (session establishment confirmation + 1 session closure) from C with win prob 1
        #
        # Average B should have 0.1 * 102 + 2 = 12.2 winning tickets

        try:
            async with create_bidirectional_channels_for_route(
                [swarm7[hop] for hop in route],
                2 * (ticket_count + 1) * ticket_price / win_prob,
                2 * ticket_price / win_prob,
            ) as channels:
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()

                # the destination should receive all the packets
                await basic_send_and_receive_packets_over_single_route(
                    ticket_count,
                    [swarm7[hop] for hop in route],
                )

                # Wait for at least some tickets to become acknowledged
                # The only certain ticket here is the one from C
                await asyncio.wait_for(
                    check_unredeemed_tickets_value(
                        swarm7[relay],
                        statistics_before.unredeemed_value + ticket_price,
                    ),
                    30.0,
                )
                await asyncio.sleep(5)  # Wait some more for the rest to become acknowledged if any

                # the value of redeemable tickets on the relay should not go above the given threshold
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                new_tickets_value = ticket_statistics.unredeemed_value - statistics_before.unredeemed_value
                winning_count = ticket_statistics.winning_count - statistics_before.winning_count
                assert new_tickets_value > Balance.zero("wxHOPR")

                # ticket_count + 2 tickets are sent with win_prob < 1 (A), and 2 are sent with win_prob = 1 (C)
                assert abs((winning_count - 2) - (ticket_count + 2) * win_prob) <= win_ticket_tolerance * (
                    ticket_count + 2
                )

                # Redeem only on the channel incoming from the Entry
                assert await swarm7[relay].api.channel_redeem_tickets(channels.fwd_channels[0].id)

                # The only 2 unredeemed tickets are on the return channel (establishment and closure)
                await asyncio.wait_for(check_unredeemed_tickets_value_max(swarm7[relay], 2 * ticket_price), 30.0)

                # The redeemed ticket value must be the new value minus the 2 tickets on the return channel
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                assert (
                    ticket_statistics.redeemed_value - statistics_before.redeemed_value
                    == new_tickets_value - 2 * ticket_price
                )

        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, Decimal("1"), base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [
            [
                *random.sample(nodes_with_lower_outgoing_win_prob(), 1),
                *random.sample(barebone_nodes(), 2),
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_should_reject_unredeemed_tickets_with_lower_win_prob_when_min_bound_increases(
        self, route: list[str], swarm7: dict[str, Node], base_port: int
    ):
        assert len(route) == len(set(route))  # checks that all elements of route are unique. Should never fail

        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 100
        win_prob = Decimal("0.1")
        win_ticket_tolerance = Decimal("0.3")
        relay = route[1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)
        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob), 10.0)

        try:
            async with create_bidirectional_channels_for_route(
                [swarm7[hop] for hop in route],
                2 * (ticket_count + 1) * ticket_price / win_prob,
                2 * ticket_price / win_prob,
            ):
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_before = statistics_before.unredeemed_value
                rejected_value_before = statistics_before.rejected_value

                # the destination should receive all the packets
                await basic_send_and_receive_packets_over_single_route(
                    ticket_count,
                    [swarm7[hop] for hop in route],
                )

                # Wait until the tickets are acknowledged,
                # we cannot do it here using wait_for(check_unredeemed_tickets_value(x)), because the
                # winning probability would make the awaited value non-deterministic
                await asyncio.sleep(5)

                # the value of redeemable tickets on the relay should not go above the given threshold
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_1 = ticket_statistics.unredeemed_value
                winning_count = ticket_statistics.winning_count - statistics_before.winning_count
                rejected_value = ticket_statistics.rejected_value
                assert unredeemed_value_1 - unredeemed_value_before > Balance.zero("wxHOPR")
                assert abs(winning_count - ticket_count * win_prob) <= win_ticket_tolerance * ticket_count
                assert rejected_value - rejected_value_before == Balance.zero("wxHOPR")

                # Now if we increase the minimum winning probability, the relayer should
                # reject all the unredeemed tickets
                set_minimum_winning_probability_in_network(private_key, win_prob * 2, base_port)
                await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob * 2), 10.0)

                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_2 = ticket_statistics.unredeemed_value
                rejected_value = ticket_statistics.rejected_value

                # We need to subtract 2 tickets which were not rejected because they were sent
                # for the RP from the exit to the relay during the session establishment and closure.
                # Therefore, they have win_prob = 1 and do not get rejected
                # by increasing the winning probability threshold.
                assert unredeemed_value_2 - unredeemed_value_before == 2 * ticket_price
                assert rejected_value - rejected_value_before == unredeemed_value_1 - 2 * ticket_price

        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, Decimal("1.0"), base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [
            [
                *random.sample(nodes_with_lower_outgoing_win_prob(), 1),
                *random.sample(barebone_nodes(), 3),
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_should_relay_with_increased_win_prob(self, route, swarm7: dict[str, Node], base_port: int):
        assert len(route) == len(set(route))  # checks that all elements of route are unique. Should never fail

        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 100
        win_prob = Decimal("0.1")
        win_ticket_tolerance = Decimal("0.3")

        relay_1 = route[1]
        relay_2 = route[2]

        private_key = load_private_key(ANVIL_CONFIG_FILE)
        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay_1], win_prob), 10.0)

        try:
            async with create_bidirectional_channels_for_route(
                [swarm7[hop] for hop in route],
                2 * (ticket_count + 1) * ticket_price / win_prob,
                2 * ticket_price / win_prob,
            ):
                # ensure ticket stats are what we expect before starting
                statistics_before_1 = await swarm7[relay_1].api.get_tickets_statistics()
                unredeemed_value_before_1 = statistics_before_1.unredeemed_value

                statistics_before_2 = await swarm7[relay_2].api.get_tickets_statistics()
                unredeemed_value_before_2 = statistics_before_2.unredeemed_value

                # the destination should receive all the packets
                await basic_send_and_receive_packets_over_single_route(
                    ticket_count,
                    [swarm7[hop] for hop in route],
                )

                # since the first relay sends tickets with win probability = 1,
                # the second relay must get all the tickets as winning
                await asyncio.wait_for(
                    check_winning_tickets_count(swarm7[relay_2], ticket_count),
                    30.0,
                )
                ticket_statistics = await swarm7[relay_2].api.get_tickets_statistics()
                unredeemed_value_2 = ticket_statistics.unredeemed_value
                assert unredeemed_value_2 - unredeemed_value_before_2 > Balance.zero("wxHOPR")

                # the value of redeemable tickets on the first relay should not go above the given threshold
                ticket_statistics = await swarm7[relay_1].api.get_tickets_statistics()
                unredeemed_value_1 = ticket_statistics.unredeemed_value
                winning_count_1 = ticket_statistics.winning_count - statistics_before_1.winning_count
                assert unredeemed_value_1 - unredeemed_value_before_1 > Balance.zero("wxHOPR")
                assert abs(winning_count_1 - ticket_count * win_prob) <= win_ticket_tolerance * ticket_count
        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, Decimal("1.0"), base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src, dest",
        [random.sample(barebone_nodes(), 2) for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    )
    @pytest.mark.parametrize(
        "mid",
        [random.choice(nodes_with_lower_outgoing_win_prob()) for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    )
    async def test_hoprd_should_relay_packets_with_higher_than_min_win_prob(
        self, src: str, mid: str, dest: str, swarm7: dict[str, Node], base_port: int
    ):
        route = [src, mid, dest]
        assert len(route) == len(set(route))  # checks that all elements of route are unique. Should never fail

        ticket_price = await get_ticket_price(swarm7[route[0]])
        ticket_count = 100
        win_prob = Decimal("0.1")

        relay = route[1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)
        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob), 10.0)

        try:
            async with create_bidirectional_channels_for_route(
                [swarm7[hop] for hop in route],
                2 * (ticket_count + 1) * ticket_price / win_prob,
                2 * ticket_price / win_prob,
            ):
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_before = statistics_before.unredeemed_value
                rejected_value_before = statistics_before.rejected_value

                # the destination should receive all the packets
                await basic_send_and_receive_packets_over_single_route(
                    ticket_count,
                    [swarm7[hop] for hop in route],
                )

                # in this case, the relay has tickets for all the packets, because the source sends
                # them with win prob = 1
                await asyncio.wait_for(
                    check_unredeemed_tickets_value(
                        swarm7[relay], unredeemed_value_before + (ticket_count + 4) * ticket_price
                    ),
                    30.0,
                )

                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                rejected_value = ticket_statistics.rejected_value

                # 4 additional tickets come from the Session establishment and closure
                assert ticket_statistics.winning_count - statistics_before.winning_count == ticket_count + 4
                assert rejected_value - rejected_value_before == Balance.zero("wxHOPR")

                # at this point the tickets become neglected, since the channel will be closed
        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, Decimal("1.0"), base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [
            [
                *random.sample(nodes_with_lower_outgoing_win_prob(), 1),
                *random.sample(barebone_nodes(), 2),
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_should_not_accept_tickets_with_lower_than_min_win_prob(
        self, route: list[str], swarm7: dict[str, Node]
    ):
        assert len(route) == len(set(route))  # checks that all elements of route are unique. Should never fail

        ticket_price = await get_ticket_price(swarm7[route[0]])
        win_prob = Decimal("0.1")

        src, relay, dest = route

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route],
            3 * ticket_price / win_prob,
            2 * ticket_price / win_prob,
        ):
            # ensure ticket stats are what we expect before starting
            statistics_before = await swarm7[relay].api.get_tickets_statistics()
            unredeemed_value_before = statistics_before.unredeemed_value
            rejected_value_before = statistics_before.rejected_value

            was_active = False
            try:
                async with HoprSession(
                    Protocol.UDP,
                    swarm7[src],
                    swarm7[dest],
                    fwd_path={"IntermediatePath": [swarm7[relay].address]},
                    return_path={"IntermediatePath": [swarm7[relay].address]},
                    use_response_buffer=None,
                ):
                    was_active = True
            except Exception as e:
                logging.debug(f"session establishment failed as expected: {e}")

                # wait until the relay rejects the session establishment packet
                await asyncio.wait_for(
                    check_rejected_tickets_value(swarm7[relay], rejected_value_before + ticket_price / win_prob),
                    30.0,
                )

                # unredeemed value should not change on the relay
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                assert ticket_statistics.unredeemed_value == unredeemed_value_before
                assert ticket_statistics.winning_count == statistics_before.winning_count

            assert was_active is False
