import asyncio
import logging
import random
import re

import pytest

from sdk.python.api import Protocol
from sdk.python.api.balance import Balance
from sdk.python.api.channelstatus import ChannelStatus
from sdk.python.api.request_objects import SessionCapabilitiesBody
from sdk.python.api.response_objects import Metrics
from sdk.python.localcluster.constants import OPEN_CHANNEL_FUNDING_VALUE_HOPR
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes, default_nodes, random_distinct_pairs_from
from .utils import (
    PARAMETERIZED_SAMPLE_SIZE,
    TICKET_AGGREGATION_THRESHOLD,
    HoprSession,
    basic_send_and_receive_packets,
    basic_send_and_receive_packets_over_single_route,
    check_all_tickets_redeemed,
    check_native_balance_below,
    check_rejected_tickets_value,
    check_safe_balance,
    check_unredeemed_tickets_value,
    create_bidirectional_channels_for_route,
    create_channel,
    get_ticket_price,
    shuffled,
)


async def assert_channel_statuses(api):
    open_channels = await api.all_channels(include_closed=False)
    open_and_closed_channels = await api.all_channels(include_closed=True)

    assert len(open_and_closed_channels.all) > 0, "More than 0 channels are present"
    assert len(open_and_closed_channels.all) >= len(open_channels.all), "Open and closed channels should be present"

    assert all(
        c.status in [ChannelStatus.Closed, ChannelStatus.PendingToClose] for c in open_and_closed_channels.all
    ), "All channels must be closed or closing"


@pytest.mark.usefixtures("swarm7_reset")
class TestIntegrationWithSwarm:
    # NOTE: this test is first, ensuring that all tests following it have ensured connectivity and
    # correct ticket price from api
    @pytest.mark.asyncio
    async def test_hoprd_swarm_connectivity(self, swarm7: dict[str, Node]):
        async def check_all_connected(me: Node, others: list[str]):
            others2 = set(others)
            while True:
                current_peers = set([x.address for x in await me.api.peers()])
                if current_peers.intersection(others) == others2:
                    break
                else:
                    assert current_peers.intersection(others2) == others2
                    await asyncio.sleep(0.5)

        await asyncio.gather(
            *[
                asyncio.wait_for(
                    check_all_connected(swarm7[k], [swarm7[v].address for v in barebone_nodes() if v != k]), 60.0
                )
                for k in barebone_nodes()
            ]
        )

    @pytest.mark.asyncio
    async def test_hoprd_protocol_check_balances_without_prior_tests(self, swarm7: dict[str, Node]):
        for node in swarm7.values():
            addr = await node.api.addresses()
            assert re.match("^0x[0-9a-fA-F]{40}$", addr.native) is not None
            balances = await node.api.balances()
            assert balances.native > Balance.zero("xDai")
            assert balances.safe_hopr > Balance.zero("wxHOPR")

    @pytest.mark.asyncio
    @pytest.mark.parametrize("src, dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
    async def test_hoprd_ping_should_work_between_nodes_in_the_same_network(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        response = await swarm7[src].api.ping(swarm7[dest].address)

        assert response is not None
        # Zero-roundtrip (in ms precision) can happen on fast local setups
        assert int(response.latency) >= 0

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hoprd_ping_to_self_should_fail(self, peer: str, swarm7: dict[str, Node]):
        response = await swarm7[peer].api.ping(swarm7[peer].address)

        assert response is None, "Pinging self should fail"

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="Test not yet implemented")
    async def test_hoprd_ping_should_not_be_able_to_ping_nodes_not_present_in_the_registry(
        self,
        swarm7: dict[str, Node],
    ):
        """
        # log "Node 7 should not be able to talk to Node 1 (Node 7 is not in the register)"
        # result=$(ping "${api7}" ${addr1} "TIMEOUT")
        # log "-- ${result}"

        # log "Node 1 should not be able to talk to Node 7 (Node 7 is not in the register)"
        # result=$(ping "${api1}" ${addr7} "TIMEOUT")
        # log "-- ${result}"
        """
        assert True

    @pytest.mark.asyncio
    @pytest.mark.parametrize("src, dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
    async def test_hoprd_should_be_able_to_send_0_hop_messages_without_open_channels(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)

        await basic_send_and_receive_packets(
            message_count, src=swarm7[src], dest=swarm7[dest], fwd_path={"Hops": 0}, return_path={"Hops": 0}
        )

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_hoprd_api_channel_should_register_fund_increase_using_fund_endpoint(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        # convert HOPR to weiHOPR
        hopr_amount = OPEN_CHANNEL_FUNDING_VALUE_HOPR
        ticket_price = await get_ticket_price(swarm7[src])

        async with create_channel(swarm7[src], swarm7[dest], funding=ticket_price) as channel:
            balance_before = await swarm7[src].api.balances()
            channel_before = await swarm7[src].api.get_channel(channel.id)
            logging.debug(f"balance_before: {balance_before}, channel_before: {channel_before}")

            assert await swarm7[src].api.fund_channel(channel.id, hopr_amount)

            channel_after = await swarm7[src].api.get_channel(channel.id)

            # The updated channel balance is visible immediately
            assert channel_after.balance - channel_before.balance == hopr_amount

            # Wait until the safe balance has decreased
            await asyncio.wait_for(
                check_safe_balance(swarm7[src], balance_before.safe_hopr - hopr_amount),
                20.0,
            )

            # Safe allowance can be checked too at this point
            balance_after = await swarm7[src].api.balances()
            logging.debug(f"balance_after: {balance_after}")

            assert balance_before.safe_hopr_allowance - balance_after.safe_hopr_allowance == hopr_amount
            await asyncio.wait_for(check_native_balance_below(swarm7[src], balance_before.native), 20.0)

        await assert_channel_statuses(swarm7[src].api)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,mid,dest", [tuple(shuffled(barebone_nodes())[:3]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_reset_ticket_statistics_from_metrics(self, src: str, mid: str, dest: str, swarm7: dict[str, Node]):
        def count_metrics(metrics: Metrics):
            types = ["neglected", "redeemed", "rejected"]
            return sum(metrics.hopr_tickets_incoming_statistics.get(t, 0) for t in types)

        ticket_price = await get_ticket_price(swarm7[src])

        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[mid], swarm7[dest]],
            3 * ticket_price,
            2 * ticket_price,
        ):
            await basic_send_and_receive_packets_over_single_route(
                1,
                [swarm7[src], swarm7[mid], swarm7[dest]],
            )

        assert count_metrics(await swarm7[mid].api.metrics()) != 0

        await swarm7[mid].api.reset_tickets_statistics()

        assert count_metrics(await swarm7[mid].api.metrics()) == 0

        await assert_channel_statuses(swarm7[src].api)

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "src,mid,dest", [tuple(shuffled(barebone_nodes())[:3]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    )
    async def test_hoprd_should_reject_relaying_a_message_when_the_channel_is_out_of_funding(
        self, src: str, mid: str, dest: str, swarm7: dict[str, Node]
    ):
        ticket_price = await get_ticket_price(swarm7[src])
        unredeemed_value_before = (await swarm7[mid].api.get_tickets_statistics()).unredeemed_value
        rejected_value_before = (await swarm7[mid].api.get_tickets_statistics()).rejected_value

        message_count = 3

        # The forward channel has funding for the Session establishment message, and message_count more messages
        # The return channel has only funding for the Session Establishment message
        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[mid], swarm7[dest]],
            (message_count + 1) * ticket_price,
            ticket_price,
        ):
            async with HoprSession(
                Protocol.UDP,
                swarm7[src],
                swarm7[dest],
                {"IntermediatePath": [swarm7[mid].address]},
                {"IntermediatePath": [swarm7[mid].address]},
                capabilities=SessionCapabilitiesBody(segmentation=True, no_delay=True),
                use_response_buffer=None,
            ) as session:
                # Unredeemed value at the relay must be greater by 2 tickets (session establishment messages)
                await asyncio.wait_for(
                    check_unredeemed_tickets_value(swarm7[mid], unredeemed_value_before + 2 * ticket_price), 5.0
                )
                unredeemed_value_before = (await swarm7[mid].api.get_tickets_statistics()).unredeemed_value
                logging.debug(f"Unredeemed value before the test: {unredeemed_value_before}")

                with session.client_socket() as s:
                    s.settimeout(5)
                    # These messages will pass through
                    for i in range(message_count):
                        message = f"#{i}".ljust(session.mtu)
                        s.sendto(message.encode(), ("127.0.0.1", session.listen_port))

                        # Each packet increases the unredeemed value
                        await asyncio.wait_for(
                            check_unredeemed_tickets_value(
                                swarm7[mid], unredeemed_value_before + (i + 1) * ticket_price
                            ),
                            5.0,
                        )
                        logging.debug(f"Message #{i + 1} accounted for")

                    # This additional message is not covered
                    s.sendto(
                        "This message is not covered".ljust(session.mtu).encode(),
                        ("127.0.0.1", session.listen_port),
                    )

                    # And it should be seen as rejected eventually
                    await asyncio.wait_for(
                        check_rejected_tickets_value(swarm7[mid], rejected_value_before + ticket_price), 10.0
                    )

                # Redeem all remaining tickets
                assert await swarm7[mid].api.tickets_redeem()
                await asyncio.wait_for(check_all_tickets_redeemed(swarm7[mid]), 30.0)

        await assert_channel_statuses(swarm7[src].api)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
    async def test_hoprd_should_be_able_to_open_and_close_channel_without_tickets(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        async with create_channel(swarm7[src], swarm7[dest], OPEN_CHANNEL_FUNDING_VALUE_HOPR):
            # the context manager handles opening and closing of the channel with verification,
            # using counter-party address
            assert True

        await assert_channel_statuses(swarm7[src].api)

    # generate a 1-hop route with a node using strategies in the middle
    @pytest.mark.asyncio
    @pytest.mark.skip(reason="ticket aggregation is not implemented as a session protocol yet")
    @pytest.mark.parametrize(
        "route",
        [
            [
                random.sample(barebone_nodes(), 1)[0],
                random.sample(default_nodes(), 1)[0],
                random.sample(barebone_nodes(), 1)[0],
            ]
            for _ in range(PARAMETERIZED_SAMPLE_SIZE)
        ],
    )
    async def test_hoprd_default_strategy_automatic_ticket_aggregation_and_redeeming(
        self, route, swarm7: dict[str, Node]
    ):
        ticket_count = int(TICKET_AGGREGATION_THRESHOLD)
        src = route[0]
        mid = route[1]
        dest = route[-1]
        ticket_price = await get_ticket_price(swarm7[src])
        aggregated_ticket_price = TICKET_AGGREGATION_THRESHOLD * ticket_price

        # Create a channel from src to mid, mid to dest does not need a channel
        async with create_bidirectional_channels_for_route(
            [swarm7[src], swarm7[mid], swarm7[dest]], ticket_count * ticket_price, ticket_price
        ):
            statistics_before = await swarm7[mid].api.get_tickets_statistics()
            assert statistics_before is not None

            await basic_send_and_receive_packets_over_single_route(
                ticket_count,
                [swarm7[src], swarm7[mid], swarm7[dest]],
            )

            # monitor that the node aggregates and redeems tickets until the aggregated value is reached
            async def check_aggregate_and_redeem_tickets(node: Node):
                while True:
                    statistics_now = await node.api.get_tickets_statistics()
                    assert statistics_now is not None

                    redeemed_value_diff = statistics_now.redeemed_value - statistics_before.redeemed_value
                    logging.debug(
                        f"redeemed_value diff: {redeemed_value_diff} |"
                        + f"before: {statistics_before.redeemed_value} |"
                        + f"now: {statistics_now.redeemed_value} |"
                        + f"target: {aggregated_ticket_price}"
                    )

                    # break out of the loop if the aggregated value is reached
                    if redeemed_value_diff >= aggregated_ticket_price:
                        break
                    else:
                        await asyncio.sleep(0.1)

            await asyncio.wait_for(check_aggregate_and_redeem_tickets(swarm7[mid]), 60.0)

        await assert_channel_statuses(swarm7[src].api)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hoprd_check_native_withdraw(self, peer, swarm7: dict[str, Node]):
        before_balance = (await swarm7[peer].api.balances()).native
        assert before_balance > Balance.zero("xDai")

        # Withdraw some native balance into the Safe address
        amount = before_balance / 10
        await swarm7[peer].api.withdraw(amount, swarm7[peer].safe_address)

        await asyncio.wait_for(check_native_balance_below(swarm7[peer], before_balance - amount), 60.0)

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hoprd_check_ticket_price_is_default(self, peer, swarm7: dict[str, Node]):
        price = await swarm7[peer].api.ticket_price()

        assert isinstance(price.value, Balance)
        assert price.value > Balance.zero("wxHOPR")

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hoprd_check_api_version(self, peer, swarm7: dict[str, Node]):
        version = await swarm7[peer].api.api_version()

        assert re.match(r"^\d+\.\d+\.\d+$", version) is not None, "Version should be in the format X.Y.Z"

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
    async def test_hoprd_configuration_endpoint(self, peer, swarm7: dict[str, Node]):
        cfg = await swarm7[peer].api.config()

        strategies_field = {k: v for d in cfg.strategies for k, v in d.items()}

        assert strategies_field["Aggregating"]["unrealized_balance_ratio"] == 0.9
