import itertools
import json
import os
import random
import subprocess
from contextlib import asynccontextmanager, AsyncExitStack

import asyncio
import pytest
from conftest import (
    NODES,
    DEFAULT_API_TOKEN,
    OPEN_CHANNEL_FUNDING_VALUE,
    TICKET_AGGREGATION_THRESHOLD,
    TICKET_PRICE_PER_HOP,
)


PARAMETERIZED_SAMPLE_SIZE = 1  # if os.getenv("CI", default="false") == "false" else 3
AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP
MULTIHOP_MESSAGE_SEND_TIMEOUT = 30.0
CHECK_RETRY_INTERVAL = 0.5


def shuffled(coll):
    random.shuffle(coll)
    return coll


def default_nodes():
    """All nodes within the same network as specified in the swarm7 fixture"""
    return list(NODES.keys())[:4]


def passive_node():
    """A node that uses no strategy"""
    return "5"


@asynccontextmanager
async def create_channel(src, dest, funding: int):
    channel = await src["api"].open_channel(dest["address"], str(int(funding)))
    assert channel is not None
    await asyncio.wait_for(check_channel_status(src, dest, status="Open"), 10.0)
    try:
        yield channel
    finally:
        assert await src["api"].close_channel(channel)
        await asyncio.wait_for(check_channel_status(src, dest, status="PendingToClose"), 10.0)

        # need to wait some more time until closure time has passed
        await asyncio.sleep(10)

        assert await src["api"].close_channel(channel)
        await asyncio.wait_for(check_channel_status(src, dest, status="Closed"), 10.0)


async def get_channel(src, dest, include_closed=False):
    open_channels = await src["api"].all_channels(include_closed=include_closed)
    channels = [
        oc
        for oc in open_channels.all
        if oc.source_address == src["address"] and oc.destination_address == dest["address"]
    ]

    return channels[0] if len(channels) > 0 else None


async def check_channel_status(src, dest, status):
    assert status in ["Open", "PendingToClose", "Closed"]
    while True:
        channel = await get_channel(src, dest, include_closed=False)
        if channel is not None and channel.status == status:
            break
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_outgoing_channel_closed(src, channel_id: str):
    while True:
        channel = await src["api"].get_channel(channel_id)
        if channel is not None and channel.status == "Closed":
            break
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)


async def check_received_packets(receiver, expected_packets, sort=True):
    received = []

    while len(received) != len(expected_packets):
        packet = await receiver["api"].messages_pop()
        if packet is not None:
            received.append(packet.body)
        else:
            await asyncio.sleep(CHECK_RETRY_INTERVAL)

    if sort:
        received.sort()

    assert received == expected_packets


async def check_all_tickets_redeemed(src):
    while (await src["api"].get_tickets_statistics()).unredeemed > 0:
        await asyncio.sleep(CHECK_RETRY_INTERVAL)


def random_distinct_pairs_from(values: list, count: int):
    return random.sample([(left, right) for left, right in itertools.product(values, repeat=2) if left != right], count)


# NOTE: this test is first, ensuring that all tests following it have ensured connectivity
@pytest.mark.asyncio
async def test_hoprd_swarm_connectivity(swarm7):
    async def check_all_connected(me, others: list):
        others = set(others)
        while True:
            current_peers = set([x["peer_id"] for x in await me["api"].peers()])
            if current_peers.intersection(others) == others:
                break
            else:
                assert current_peers.intersection(others) == others
                await asyncio.sleep(0.5)

    await asyncio.gather(
        *[
            asyncio.wait_for(
                check_all_connected(swarm7[k], [swarm7[v]["peer_id"] for v in default_nodes() if v != k]), 30.0
            )
            for k in default_nodes()
        ]
    )


def test_hoprd_protocol_post_fixture_setup_tests(swarm7):
    """
    Tests run in bash file that more or less need to be run in the future python fixture.
    """
    with open("/tmp/hopr-smoke-test-anvil.cfg") as f:
        data = json.load(f)

    anvil_private_key = data["private_keys"][0]

    env_vars = os.environ.copy()
    env_vars.update(
        {
            "HOPRD_API_TOKEN": f"{DEFAULT_API_TOKEN}",
            "PRIVATE_KEY": f"{anvil_private_key}",
        }
    )

    nodes_api_as_str = " ".join(list(map(lambda x: f"\"localhost:{x['api_port']}\"", swarm7.values())))

    log_file_path = f"/tmp/hopr-smoke-{__name__}.log"
    subprocess.run(
        [
            "bash",
            "-o",
            "pipefail",
            "-c",
            f"./tests/test_after_fixture_ready.sh {nodes_api_as_str} 2>&1 | tee --append {log_file_path}",
        ],
        shell=False,
        capture_output=True,
        env=env_vars,
        # timeout=2000,
        check=True,
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
async def test_hoprd_node_should_be_able_to_alias_other_peers(peer, swarm7):
    peer_id = swarm7[random.choice(default_nodes())]["peer_id"]

    assert await swarm7[peer]["api"].aliases_set_alias("Alice", peer_id)
    assert await swarm7[peer]["api"].aliases_get_alias("Alice") == peer_id

    assert await swarm7[peer]["api"].aliases_remove_alias("Alice")
    assert await swarm7[peer]["api"].aliases_get_alias("Alice") is None


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_ping_should_work_between_nodes_in_the_same_network(src, dest, swarm7):
    response = await swarm7[src]["api"].ping(swarm7[dest]["peer_id"])

    assert response is not None
    assert int(response.latency) > 0, f"Non-0 round trip time expected, actual: '{int(response.latency)}'"


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
async def test_hoprd_ping_should_timeout_on_pinging_self(peer, swarm7):
    response = await swarm7[peer]["api"].ping(swarm7[peer]["peer_id"])

    assert response is None, f"Pinging self should produce timeout, not '{response}'"


@pytest.mark.asyncio
async def test_hoprd_ping_should_not_be_able_to_ping_nodes_in_other_network_UNFINISHED(swarm7):
    """
    # FIXME: re-enable when network check works
    # log "Node 1 should not be able to talk to Node 6 (different network id)"
    # result=$(api_ping "${api6}" ${addr1} "TIMEOUT")
    # log "-- ${result}"

    # FIXME: re-enable when network check works
    # log "Node 6 should not be able to talk to Node 1 (different network id)"
    # result=$(api_ping "${api6}" ${addr1} "TIMEOUT")
    # log "-- ${result}"
    """
    assert True


@pytest.mark.asyncio
async def test_hoprd_ping_should_not_be_able_to_ping_nodes_not_present_in_the_registry_UNFINISHED(swarm7):
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
@pytest.mark.parametrize("peer", default_nodes())
async def test_hoprd_should_not_have_unredeemed_tickets_without_sending_messages(peer, swarm7):
    statistics = await swarm7[peer]["api"].get_tickets_statistics()

    assert int(statistics.unredeemed_value) == 0
    assert int(statistics.unredeemed) == 0


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_should_be_able_to_send_0_hop_messages_without_open_channels(src, dest, swarm7):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)

    packets = [f"0 hop message #{i:08d}" for i in range(message_count)]

    for packet in packets:
        assert await swarm7[src]["api"].send_message(swarm7[dest]["peer_id"], packet, [])

    packets.sort()
    await asyncio.wait_for(check_received_packets(swarm7[dest], packets, sort=True), MULTIHOP_MESSAGE_SEND_TIMEOUT)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "src,dest", [(passive_node(), random.choice(default_nodes())) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
)
async def test_hoprd_api_channel_should_register_fund_increase_using_fund_endpoint(src, dest, swarm7):
    hopr_amount = OPEN_CHANNEL_FUNDING_VALUE

    async with create_channel(swarm7[src], swarm7[dest], funding=TICKET_PRICE_PER_HOP) as channel:
        balance_before = await swarm7[src]["api"].balances()

        assert await swarm7[src]["api"].channels_fund_channel(channel, hopr_amount)

        balance_after = await swarm7[src]["api"].balances()

        assert int(balance_before.safe_hopr) - int(balance_after.safe_hopr) == int(hopr_amount)
        assert int(balance_before.safe_hopr_allowance) - int(balance_after.safe_hopr_allowance) == int(hopr_amount)
        assert int(balance_after.native) < int(balance_before.native)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "src,dest", [(random.choice(default_nodes()), passive_node()) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
)
async def test_hoprd_api_should_redeem_tickets_in_channel_using_redeem_endpoint(src, dest, swarm7):
    message_count = 2

    async with create_channel(swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP) as channel:
        packets = [f"Channel redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]

        for packet in packets:
            assert await swarm7[src]["api"].send_message(swarm7[src]["peer_id"], packet, [swarm7[dest]["peer_id"]])

        packets.sort()
        await asyncio.wait_for(check_received_packets(swarm7[src], packets, sort=True), 30.0)

        statistics = await swarm7[dest]["api"].get_tickets_statistics()
        assert statistics.unredeemed == message_count

        async def channel_redeem_tickets(api, channel):
            while True:
                if await api.channel_redeem_tickets(channel):
                    break
                else:
                    await asyncio.sleep(0.5)

        await asyncio.wait_for(channel_redeem_tickets(swarm7[dest]["api"], channel), 20.0)

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

        assert await swarm7[dest]["api"].channel_get_tickets(channel) == []


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "src,dest", [(passive_node(), random.choice(default_nodes())) for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
)
async def test_hoprd_should_fail_sending_a_message_when_the_channel_is_out_of_funding(src, dest, swarm7):
    """
    # FIXME: The following part can be enabled once incoming channel closure is
    # implemented.
    #
    # need to close the incoming side to not have to wait for the closure timeout
    # api_close_channel "${second_node_id}" "${node_id}" "${second_node_api}" "${node_addr}" "incoming"

    # only fund for 2 tickets
    # channel_info=$(api_open_channel "${node_id}" "${second_node_id}" "${node_api}" "${second_node_addr}" "200")

    # need to wait a little to allow the other side to index the channel open event
    # sleep 10
    # api_get_tickets_in_channel ${second_node_api} ${channel_id} "TICKETS_NOT_FOUND"
    # for i in `seq 1 ${generated_tickets}`; do
    #   log "PendingBalance in channel: Node ${node_id} send 1 hop message to self via node ${second_node_id}"
    #   api_send_message "${node_api}" "${msg_tag}" "${peer_id}" \
    #       "pendingbalance: hello, world 1 self" "${second_peer_id}"
    # done

    # seems like there's slight delay needed for tickets endpoint to return up to date tickets, \
    #       probably because of blockchain sync delay
    # sleep 5

    # ticket_amount=$(api_get_tickets_in_channel ${second_node_api} ${channel_id} | jq '. | length')
    # if [[ "${ticket_amount}" != "${generated_tickets}" ]]; then
    #   msg "PendingBalance: Ticket amount ${ticket_amount} is different than expected ${generated_tickets}"
    #   exit 1
    # fi

    # api_redeem_tickets_in_channel ${second_node_api} ${channel_id}
    # sleep 5
    # api_get_tickets_in_channel ${second_node_api} ${channel_id} "TICKETS_NOT_FOUND"
    # api_close_channel "${node_id}" "${second_node_id}" "${node_api}" "${second_node_addr}" "outgoing"
    """

    message_count = 2

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(swarm7[i[0]], swarm7[i[1]], funding=message_count * TICKET_PRICE_PER_HOP)
                )
                for i in [[src, dest], [dest, src]]
            ]
        )

        packets = [f"Channel agg and redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]

        for packet in packets:
            assert await swarm7[src]["api"].send_message(swarm7[src]["peer_id"], packet, [swarm7[dest]["peer_id"]])

        packets.sort()
        await asyncio.wait_for(check_received_packets(swarm7[src], packets, sort=True), 30.0)

        # this message has no funding in the channel, so it should fail
        assert await swarm7[src]["api"].send_message(swarm7[src]["peer_id"], packet, [swarm7[dest]["peer_id"]]) is False

        statistics = await swarm7[dest]["api"].get_tickets_statistics()
        assert int(statistics.unredeemed) == message_count

        assert await swarm7[dest]["api"].tickets_redeem()

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_1_hop_to_self_scenario(src, dest, swarm7):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)

    async with create_channel(swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP):
        packets = [f"1 hop message to self: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]

        for packet in packets:
            assert await swarm7[src]["api"].send_message(swarm7[src]["peer_id"], packet, [swarm7[dest]["peer_id"]])

        packets.sort()
        await asyncio.wait_for(check_received_packets(swarm7[src], packets, sort=True), 30.0)

        statistics = await swarm7[dest]["api"].get_tickets_statistics()
        assert (statistics.redeemed + statistics.unredeemed) > 0

        assert await swarm7[dest]["api"].tickets_redeem()

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
@pytest.mark.skip(reason="Failing due to a bug in the application")
async def test_hoprd_should_aggregate_and_redeem_tickets_in_channel_on_api_request(src, dest, swarm7):
    message_count = 2

    async with create_channel(swarm7[src], swarm7[dest], funding=message_count * TICKET_PRICE_PER_HOP) as channel:
        packets = [f"Channel agg and redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]

        for packet in packets:
            assert await swarm7[src]["api"].send_message(swarm7[src]["peer_id"], packet, [swarm7[dest]["peer_id"]])

        packets.sort()
        await asyncio.wait_for(check_received_packets(swarm7[src], packets, sort=True), 30.0)

        statistics = await swarm7[dest]["api"].get_tickets_statistics()
        assert int(statistics.unredeemed) == message_count

        async def channel_aggregate_and_redeem_tickets(api, channel):
            while True:
                if await api.channels_aggregate_tickets(channel):
                    break
                else:
                    await asyncio.sleep(0.5)

        await asyncio.wait_for(channel_aggregate_and_redeem_tickets(swarm7[dest]["api"], channel), 20.0)

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)

        assert await swarm7[dest]["api"].channel_get_tickets(channel) == []

        statistics = await swarm7[dest]["api"].get_tickets_statistics()
        assert int(statistics.redeemed_value) == message_count * TICKET_PRICE_PER_HOP
        assert int(statistics.redeemed) == 1


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(default_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_general_n_hop(route, swarm7):
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

        for packet in packets:
            assert await swarm7[route[0]]["api"].send_message(
                swarm7[route[-1]]["peer_id"], packet, [swarm7[x]["peer_id"] for x in route[1:-1]]
            )

        packets.sort()
        await asyncio.wait_for(
            check_received_packets(swarm7[route[-1]], packets, sort=True), MULTIHOP_MESSAGE_SEND_TIMEOUT
        )

        statistics = await swarm7[route[1]]["api"].get_tickets_statistics()
        assert (statistics.redeemed + statistics.unredeemed) > 0

        assert await swarm7[route[1]]["api"].tickets_redeem()

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[route[1]]), 120.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("route", [shuffled(default_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_should_be_able_to_close_open_channels_with_unredeemed_tickets(route, swarm7):
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

        for packet in packets:
            assert await swarm7[route[0]]["api"].send_message(
                swarm7[route[-1]]["peer_id"], packet, [swarm7[route[1]]["peer_id"]]
            )

        packets.sort()
        await asyncio.wait_for(
            check_received_packets(swarm7[route[-1]], packets, sort=True), MULTIHOP_MESSAGE_SEND_TIMEOUT
        )

        statistics = await swarm7[route[1]]["api"].get_tickets_statistics()
        assert statistics.unredeemed > 0

        # NOTE: will be closed on context manager exit


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_should_be_able_to_open_and_close_channel_without_tickets(src, dest, swarm7):
    async with create_channel(swarm7[src], swarm7[dest], OPEN_CHANNEL_FUNDING_VALUE):
        # the context manager handles opening and closing of the channel with verification
        assert True


@pytest.mark.asyncio
@pytest.mark.parametrize("route", [shuffled(default_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_strategy_automatic_ticket_aggregation_and_redeeming(route, swarm7):
    ticket_count = int(TICKET_AGGREGATION_THRESHOLD)

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=ticket_count * TICKET_PRICE_PER_HOP)
                )
                for i in range(len(route) - 1)
            ]
        )

        statistics_before = await swarm7[route[1]]["api"].get_tickets_statistics()

        packets = [f"Ticket aggregation test: #{i:08d}" for i in range(ticket_count)]

        for packet in packets:
            assert await swarm7[route[0]]["api"].send_message(
                swarm7[route[-1]]["peer_id"], packet, [swarm7[route[1]]["peer_id"]]
            )

        packets.sort()
        await asyncio.wait_for(check_received_packets(swarm7[route[-1]], packets, sort=True), 30.0)

        async def aggregate_and_redeem_tickets():
            while True:
                statistics_after = await swarm7[route[1]]["api"].get_tickets_statistics()
                redeemed_value = int(statistics_after.redeemed_value) - int(statistics_before.redeemed_value)
                redeemed_ticket_count = statistics_after.redeemed - statistics_before.redeemed

                if redeemed_value >= AGGREGATED_TICKET_PRICE:
                    break
                else:
                    await asyncio.sleep(0.5)

            assert redeemed_value >= AGGREGATED_TICKET_PRICE
            assert redeemed_ticket_count == pytest.approx(redeemed_value / AGGREGATED_TICKET_PRICE, 0.1)

        await asyncio.wait_for(aggregate_and_redeem_tickets(), 60.0)


@pytest.mark.asyncio
async def test_hoprd_sanity_check_channel_status(swarm7):
    """
    The bash integration-test.sh opens and closes channels that can be visible inside this test scope
    """
    alice_api = swarm7["1"]["api"]

    open_channels = await alice_api.all_channels(include_closed=False)
    open_and_closed_channels = await alice_api.all_channels(include_closed=True)

    assert len(open_and_closed_channels.all) >= len(open_channels.all), "Open and closed channels should be present"

    statuses = [c.status for c in open_and_closed_channels.all]
    assert "Closed" in statuses or "PendingToClose" in statuses, "Closed channels should be present"


@pytest.mark.asyncio
async def test_hoprd_strategy_UNFINISHED():
    """
    ## NOTE: strategy testing will require separate setup so commented out for now until moved
    # test_strategy_setting() {
    #   local node_api="${1}"

    #   settings=$(get_settings ${node_api})
    #   strategy=$(echo ${settings} | jq -r .strategy)
    #   [[ "${strategy}" != "passive" ]] && { msg "Default strategy should be passive, got: ${strategy}"; exit 1; }

    #   channels_count_pre=$(get_all_channels ${node_api} false | jq '.incoming | length')

    #   set_setting ${node_api} "strategy" "promiscuous"

    #   log "Waiting 100 seconds for the node to make connections to other nodes"
    #   sleep 100

    #   channels_count_post=$(get_all_channels ${node_api} false | jq '.incoming | length')
    #   [[ "${channels_count_pre}" -ge "${channels_count_post}" ]] && { msg "Node didn't open any connections by \
    #    itself even when strategy was set to promiscuous: ${channels_count_pre} !>= ${channels_count_post}"; exit 1; }
    #   echo "Strategy setting successfull"
    # }

    # test_strategy_setting ${api4}
    """
    assert True


@pytest.mark.asyncio
async def test_hoprd_check_native_withdraw_results_UNFINISHED():
    """
    # this 2 functions are runned at the end of the tests when withdraw transaction should clear on blockchain
    # and we don't have to block and wait for it
    check_native_withdraw_results() {
    local initial_native_balance="${1}"

    balances=$(api_get_balances ${api1})
    new_native_balance=$(echo ${balances} | jq -r .native)
    [[ "${initial_native_balance}" == "${new_native_balance}" ]] && \
        { msg "Native withdraw failed, pre: ${initial_native_balance}, post: ${new_native_balance}"; exit 1; }

    echo "withdraw native successful"
    }

    # checking statuses of the long running tests
    balances=$(api_get_balances ${api1})
    native_balance=$(echo ${balances} | jq -r .native)
    """
    assert True
