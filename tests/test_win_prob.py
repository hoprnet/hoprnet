import asyncio
import random

import pytest

from sdk.python.localcluster.constants import ANVIL_CONFIG_FILE, NETWORK, TICKET_PRICE_PER_HOP, CONTRACTS_DIR
from sdk.python.localcluster.node import Node
from sdk.python.localcluster.utils import load_private_key

from .conftest import barebone_nodes, nodes_with_lower_outgoing_win_prob, run_hopli_cmd
from .utils import (
    PARAMETERIZED_SAMPLE_SIZE,
    check_all_tickets_redeemed,
    check_min_incoming_win_prob_eq,
    check_rejected_tickets_value,
    create_channel,
    gen_random_tag,
    send_and_receive_packets_with_pop,
    check_unredeemed_tickets_value,
    check_winning_tickets_count,
)


def generate_anvil_endpoint(base_port: int) -> str:
    return f"http://127.0.0.1:{base_port}"


def set_minimum_winning_probability_in_network(private_key: str, win_prob: float, base_port: int):
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
        assert 0.0 <= round(win_prob.value, 5) <= 1.0

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        new_win_prob = win_prob.value / 2
        set_minimum_winning_probability_in_network(private_key, new_win_prob, base_port)

        try:
            await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[peer], new_win_prob), timeout=20.0)
        finally:
            # Restore the winning probability regardless of the outcome
            set_minimum_winning_probability_in_network(private_key, win_prob.value, base_port)

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="ticket aggregation is not implemented as a session protocol yet")
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
    async def test_hoprd_should_relay_packets_with_lower_win_prob_then_agg_and_redeem_them(
        self, route, swarm7: dict[str, Node], base_port: int
    ):
        ticket_count = 100
        win_prob = 0.1
        win_ticket_tolerance = 0.1

        src = route[0]
        relay = route[1]
        dest = route[-1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob), 10.0)

        try:
            async with create_channel(
                swarm7[src], swarm7[relay], funding=2 * ticket_count * TICKET_PRICE_PER_HOP / win_prob
            ) as channel:
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()

                # the destination should receive all the packets
                await swarm7[dest].api.messages_pop_all(None)
                packets = [f"Lower ticket win probability check: #{i:08d}" for i in range(ticket_count)]
                await send_and_receive_packets_with_pop(
                    packets, src=swarm7[src], dest=swarm7[dest], path=[swarm7[relay].peer_id]
                )

                # the value of redeemable tickets on the relay should not go above the given threshold
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                new_tickets_value = ticket_statistics.unredeemed_value - statistics_before.unredeemed_value
                winning_count = ticket_statistics.winning_count - statistics_before.winning_count
                assert new_tickets_value > 0
                assert abs(winning_count - ticket_count * win_prob) <= win_ticket_tolerance * ticket_count

                await asyncio.wait_for(swarm7[relay].api.channels_aggregate_tickets(channel.id), 20.0)

                assert await swarm7[relay].api.channel_redeem_tickets(channel.id)
                await asyncio.wait_for(check_all_tickets_redeemed(swarm7[relay]), 120.0)

                # The tickets can get successfully redeemed on that channel
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                assert ticket_statistics.unredeemed_value == 0
                assert ticket_statistics.redeemed_value - statistics_before.redeemed_value == new_tickets_value
        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, 1.0, base_port)

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
        self, route, swarm7: dict[str, Node], base_port: int
    ):
        ticket_count = 100
        win_prob = 0.1
        win_ticket_tolerance = 0.1

        src = route[0]
        relay = route[1]
        dest = route[-1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob), 10.0)

        try:
            async with create_channel(
                swarm7[src], swarm7[relay], funding=2 * ticket_count * TICKET_PRICE_PER_HOP / win_prob
            ):
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_before = statistics_before.unredeemed_value
                rejected_value_before = statistics_before.rejected_value

                # the destination should receive all the packets
                await swarm7[dest].api.messages_pop_all(None)
                packets = [f"Lowering ticket win probability check: #{i:08d}" for i in range(ticket_count)]
                await send_and_receive_packets_with_pop(
                    packets, src=swarm7[src], dest=swarm7[dest], path=[swarm7[relay].peer_id]
                )

                # the value of redeemable tickets on the relay should not go above the given threshold
                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_1 = ticket_statistics.unredeemed_value
                winning_count = ticket_statistics.winning_count - statistics_before.winning_count
                rejected_value = ticket_statistics.rejected_value
                assert unredeemed_value_1 - unredeemed_value_before > 0
                assert abs(winning_count - ticket_count * win_prob) <= win_ticket_tolerance * ticket_count
                assert rejected_value - rejected_value_before == 0

                # Now if we increase the minimum winning probability, the relayer should
                # reject all the unredeemed tickets
                set_minimum_winning_probability_in_network(private_key, win_prob * 2, base_port)
                await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob * 2), 10.0)

                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_2 = ticket_statistics.unredeemed_value
                rejected_value = ticket_statistics.rejected_value
                assert rejected_value - rejected_value_before == unredeemed_value_1
                assert unredeemed_value_2 - unredeemed_value_before == 0

        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, 1.0, base_port)

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
        ticket_count = 100
        win_prob = 0.1
        win_ticket_tolerance = 0.1

        src = route[0]
        relay_1 = route[1]
        relay_2 = route[2]
        dest = route[-1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay_1], win_prob), 10.0)

        try:
            async with create_channel(
                swarm7[src], swarm7[relay_1], funding=2 * ticket_count * TICKET_PRICE_PER_HOP / win_prob
            ):
                async with create_channel(
                    swarm7[relay_1], swarm7[relay_2], funding=2 * ticket_count * TICKET_PRICE_PER_HOP / win_prob
                ):
                    # ensure ticket stats are what we expect before starting
                    statistics_before_1 = await swarm7[relay_1].api.get_tickets_statistics()
                    unredeemed_value_before_1 = statistics_before_1.unredeemed_value

                    statistics_before_2 = await swarm7[relay_2].api.get_tickets_statistics()
                    unredeemed_value_before_2 = statistics_before_2.unredeemed_value

                    # the destination should receive all the packets
                    await swarm7[dest].api.messages_pop_all(None)
                    packets = [f"Relaying ticket win probability check: #{i:08d}" for i in range(ticket_count)]
                    await send_and_receive_packets_with_pop(
                        packets,
                        src=swarm7[src],
                        dest=swarm7[dest],
                        path=[swarm7[relay_1].peer_id, swarm7[relay_2].peer_id],
                    )

                    # since the first relay sends tickets with win probability = 1,
                    # the second relay must get all the tickets as winning
                    await asyncio.wait_for(
                        check_winning_tickets_count(swarm7[relay_2], ticket_count),
                        30.0,
                    )
                    ticket_statistics = await swarm7[relay_2].api.get_tickets_statistics()
                    unredeemed_value_2 = ticket_statistics.unredeemed_value
                    assert unredeemed_value_2 - unredeemed_value_before_2 > 0

                    # the value of redeemable tickets on the first relay should not go above the given threshold
                    ticket_statistics = await swarm7[relay_1].api.get_tickets_statistics()
                    unredeemed_value_1 = ticket_statistics.unredeemed_value
                    winning_count_1 = ticket_statistics.winning_count - statistics_before_1.winning_count
                    assert unredeemed_value_1 - unredeemed_value_before_1 > 0
                    assert abs(winning_count_1 - ticket_count * win_prob) <= win_ticket_tolerance * ticket_count
        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, 1.0, base_port)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [
            [
                *random.sample(barebone_nodes(), 1),
                *random.sample(nodes_with_lower_outgoing_win_prob(), 1),
                *random.sample(barebone_nodes(), 1),
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_should_relay_packets_with_higher_than_min_win_prob(
        self, route, swarm7: dict[str, Node], base_port: int
    ):
        ticket_count = 10
        win_prob = 0.1

        src = route[0]
        relay = route[1]
        dest = route[-1]

        private_key = load_private_key(ANVIL_CONFIG_FILE)

        set_minimum_winning_probability_in_network(private_key, win_prob, base_port)
        await asyncio.wait_for(check_min_incoming_win_prob_eq(swarm7[relay], win_prob), 10.0)

        try:
            async with create_channel(
                swarm7[src], swarm7[relay], funding=2 * ticket_count * TICKET_PRICE_PER_HOP / win_prob
            ):
                # ensure ticket stats are what we expect before starting
                statistics_before = await swarm7[relay].api.get_tickets_statistics()
                unredeemed_value_before = statistics_before.unredeemed_value
                rejected_value_before = statistics_before.rejected_value

                # the destination should receive all the packets
                await swarm7[dest].api.messages_pop_all(None)
                packets = [f"Standard ticket win probability check: #{i:08d}" for i in range(ticket_count)]
                await send_and_receive_packets_with_pop(
                    packets, src=swarm7[src], dest=swarm7[dest], path=[swarm7[relay].peer_id]
                )

                # in this case, the relay has tickets for all the packets, because the source sends them with win prob = 1
                await asyncio.wait_for(
                    check_unredeemed_tickets_value(
                        swarm7[relay], unredeemed_value_before + TICKET_PRICE_PER_HOP * ticket_count
                    ),
                    30.0,
                )

                ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
                rejected_value = ticket_statistics.rejected_value
                assert ticket_statistics.winning_count - statistics_before.winning_count == ticket_count
                assert rejected_value - rejected_value_before == 0

                # at this point the tickets become neglected, since the channel will be closed
        finally:
            # Always return winning probability to 1.0 even if the test failed
            set_minimum_winning_probability_in_network(private_key, 1.0, base_port)

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
        self, route, swarm7: dict[str, Node], base_port: int
    ):
        ticket_count = 10

        src = route[0]
        relay = route[1]
        dest = route[-1]

        async with create_channel(swarm7[src], swarm7[relay], funding=ticket_count * TICKET_PRICE_PER_HOP):
            # ensure ticket stats are what we expect before starting
            statistics_before = await swarm7[relay].api.get_tickets_statistics()
            unredeemed_value_before = statistics_before.unredeemed_value
            rejected_value_before = statistics_before.rejected_value

            # sent out all packets at from source
            await swarm7[dest].api.messages_pop_all(None)
            packets = [f"Rejected ticket win probability check: #{i:08d}" for i in range(ticket_count)]
            random_tag = gen_random_tag()

            for packet in packets:
                assert await swarm7[src].api.send_message(
                    swarm7[dest].peer_id, packet, [swarm7[relay].peer_id], random_tag
                )

            # wait until the relay rejects all the tickets
            await asyncio.wait_for(
                check_rejected_tickets_value(
                    swarm7[relay], rejected_value_before + ticket_count * TICKET_PRICE_PER_HOP
                ),
                30.0,
            )

            # unredeemed value should not change on the relay
            ticket_statistics = await swarm7[relay].api.get_tickets_statistics()
            assert ticket_statistics.unredeemed_value == unredeemed_value_before
            assert ticket_statistics.winning_count == statistics_before.winning_count

            # in this case, the destination receives nothing, because the relayer will not relay packets
            # with win prob lower than 1
            messages = await swarm7[dest].api.messages_peek_all(random_tag)
            assert messages is not None
            assert len(messages) == 0
