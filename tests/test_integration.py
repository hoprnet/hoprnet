import asyncio
import pytest
import random
import re
import string
from contextlib import AsyncExitStack

from .conftest import (
    OPEN_CHANNEL_FUNDING_VALUE_HOPR,
    RESERVED_TAG_UPPER_BOUND,
    TICKET_AGGREGATION_THRESHOLD,
    TICKET_PRICE_PER_HOP,
    barebone_nodes,
    default_nodes,
    random_distinct_pairs_from
)
from .hopr import HoprdAPI
from .node import Node

from .utils import PARAMETERIZED_SAMPLE_SIZE, balance_str_to_int, gen_random_tag, send_and_receive_packets_with_pop, \
    shuffled, create_channel, check_native_balance_below, check_safe_balance, check_all_tickets_redeemed, \
    check_unredeemed_tickets_value, check_rejected_tickets_value, check_min_incoming_win_prob_eq, \
    check_received_packets_with_peek, MULTIHOP_MESSAGE_SEND_TIMEOUT

# used by nodes to get unique port assignments
PORT_BASE = 19000

# NOTE: this test is first, ensuring that all tests following it have ensured connectivity and
# correct ticket price from api
@pytest.mark.asyncio
async def test_hoprd_swarm_connectivity(swarm7: dict[str, Node]):
    async def check_all_connected(me: Node, others: list[str]):
        others2 = set(others)
        while True:
            current_peers = set([x["peer_id"] for x in await me.api.peers()])
            if current_peers.intersection(others) == others2:
                break
            else:
                assert current_peers.intersection(others2) == others2
                await asyncio.sleep(0.5)

    await asyncio.gather(
        *[
            asyncio.wait_for(
                check_all_connected(swarm7[k], [swarm7[v].peer_id for v in barebone_nodes() if v != k]), 60.0
            )
            for k in barebone_nodes()
        ]
    )

    ticket_price = await random.choice(list(swarm7.values())).api.ticket_price()
    if ticket_price is not None:
        global TICKET_PRICE_PER_HOP, AGGREGATED_TICKET_PRICE
        TICKET_PRICE_PER_HOP = ticket_price
        AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP
    else:
        print("Could not get ticket price from API, using default value")


@pytest.mark.asyncio
async def test_hoprd_protocol_check_balances_without_prior_tests(swarm7: dict[str, Node]):
    for node in swarm7.values():
        addr = await node.api.addresses("native")
        assert re.match("^0x[0-9a-fA-F]{40}$", addr) is not None
        balances = await node.api.balances()
        native_balance = int(balances.native.split(" ")[0])
        hopr_balance = int(balances.safe_hopr.split(" ")[0])
        assert native_balance > 0
        assert hopr_balance > 0


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hoprd_node_should_be_able_to_alias_other_peers(peer: str, swarm7: dict[str, Node]):
    other_peers = barebone_nodes()
    other_peers.remove(peer)

    alice_peer_id = swarm7[random.choice(other_peers)].peer_id
    my_peer_id = swarm7[peer].peer_id
    assert alice_peer_id != my_peer_id

    assert await swarm7[peer].api.aliases_get_alias("me") == my_peer_id

    assert await swarm7[peer].api.aliases_get_alias("Alice") is None
    assert await swarm7[peer].api.aliases_set_alias("Alice", alice_peer_id) is True

    assert await swarm7[peer].api.aliases_get_alias("Alice") == alice_peer_id
    assert await swarm7[peer].api.aliases_set_alias("Alice", alice_peer_id) is False

    assert await swarm7[peer].api.aliases_remove_alias("Alice")
    assert await swarm7[peer].api.aliases_get_alias("Alice") is None


@pytest.mark.asyncio
@pytest.mark.parametrize("src, dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_ping_should_work_between_nodes_in_the_same_network(src: str, dest: str, swarm7: dict[str, Node]):
    response = await swarm7[src].api.ping(swarm7[dest].peer_id)

    assert response is not None
    assert int(response.latency) > 0, f"Non-0 round trip time expected, actual: '{int(response.latency)}'"


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hoprd_ping_to_self_should_fail(peer: str, swarm7: dict[str, Node]):
    response = await swarm7[peer].api.ping(swarm7[peer].peer_id)

    assert response is None, "Pinging self should fail"


@pytest.mark.asyncio
async def test_hoprd_ping_should_not_be_able_to_ping_nodes_not_present_in_the_registry_UNFINISHED(
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
    src: str, dest: str, swarm7: dict[str, Node]
):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)

    packets = [f"0 hop message #{i:08d}" for i in range(message_count)]
    await send_and_receive_packets_with_pop(packets, src=swarm7[src], dest=swarm7[dest], path=[])

    # Remove all messages so they do not interfere with the later tests
    await swarm7[dest].api.messages_pop_all(None)


@pytest.mark.asyncio
@pytest.mark.parametrize("src, dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_should_fail_sending_a_message_that_is_too_large(src: str, dest: str, swarm7: dict[str, Node]):
    maximum_payload_size = 500
    random_tag = gen_random_tag()

    packet = "0 hop message too large: " + "".join(
        random.choices(string.ascii_uppercase + string.digits, k=maximum_payload_size)
    )
    assert await swarm7[src].api.send_message(swarm7[dest].peer_id, packet, [], random_tag) is None


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_api_channel_should_register_fund_increase_using_fund_endpoint(
    src: str, dest: str, swarm7: dict[str, Node]
):
    hopr_amount = f"{OPEN_CHANNEL_FUNDING_VALUE_HOPR * 1e18:.0f}"  # convert HOPR to weiHOPR

    async with create_channel(swarm7[src], swarm7[dest], funding=TICKET_PRICE_PER_HOP) as channel:
        balance_before = await swarm7[src].api.balances()
        channel_before = await swarm7[src].api.get_channel(channel)

        assert await swarm7[src].api.channels_fund_channel(channel, hopr_amount)

        channel_after = await swarm7[src].api.get_channel(channel)

        # Updated channel balance is visible immediately
        assert balance_str_to_int(channel_after.balance) - balance_str_to_int(
            channel_before.balance
        ) == balance_str_to_int(hopr_amount)

        # Wait until the safe balance has decreased
        await asyncio.wait_for(
            check_safe_balance(
                swarm7[src], balance_str_to_int(balance_before.safe_hopr) - balance_str_to_int(hopr_amount)
            ),
            20.0,
        )

        # Safe allowance can be checked too at this point
        balance_after = await swarm7[src].api.balances()
        assert balance_str_to_int(balance_before.safe_hopr_allowance) - balance_str_to_int(
            balance_after.safe_hopr_allowance
        ) == balance_str_to_int(hopr_amount)

        await asyncio.wait_for(check_native_balance_below(swarm7[src], balance_str_to_int(balance_before.native)), 20.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", [tuple(shuffled(barebone_nodes())[:2]) for _ in range(PARAMETERIZED_SAMPLE_SIZE)])
async def test_hoprd_should_fail_sending_a_message_when_the_channel_is_out_of_funding(
    src: str, dest: str, swarm7: dict[str, Node]
):
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
                    create_channel(
                        swarm7[i[0]], swarm7[i[1]], funding=message_count * TICKET_PRICE_PER_HOP, close_from_dest=False
                    )
                )
                for i in [[src, dest]]
            ]
        )

        packets = [f"Channel agg and redeem on 1-hop: {src} - {dest} - {src} #{i:08d}" for i in range(message_count)]
        await send_and_receive_packets_with_pop(packets, src=swarm7[src], dest=swarm7[src], path=[swarm7[dest].peer_id])

        # this message has no funding in the channel, but it still should be sent
        assert await swarm7[src].api.send_message(
            swarm7[src].peer_id, "THIS MSG IS NOT COVERED", [swarm7[dest].peer_id]
        )

        await asyncio.wait_for(check_unredeemed_tickets_value(swarm7[dest], message_count * TICKET_PRICE_PER_HOP), 30.0)

        # we should see the last message as rejected
        await asyncio.wait_for(check_rejected_tickets_value(swarm7[dest], 1), 120.0)

        await asyncio.sleep(10)  # wait for aggregation to finish
        assert await swarm7[dest].api.tickets_redeem()

        await asyncio.wait_for(check_all_tickets_redeemed(swarm7[dest]), 120.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_hoprd_should_be_able_to_open_and_close_channel_without_tickets(
    src: str, dest: str, swarm7: dict[str, Node]
):
    async with create_channel(swarm7[src], swarm7[dest], OPEN_CHANNEL_FUNDING_VALUE_HOPR):
        # the context manager handles opening and closing of the channel with verification
        assert True


# generate a 1-hop route with a node using strategies in the middle
@pytest.mark.asyncio
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
async def test_hoprd_default_strategy_automatic_ticket_aggregation_and_redeeming(route, swarm7: dict[str, Node]):
    ticket_count = int(TICKET_AGGREGATION_THRESHOLD)
    src = route[0]
    mid = route[1]
    dest = route[-1]
    channel_funding = ticket_count * TICKET_PRICE_PER_HOP

    # create channel from src to mid, mid to dest does not need a channel
    async with create_channel(swarm7[src], swarm7[mid], funding=channel_funding) as channel:
        statistics_before = await swarm7[mid].api.get_tickets_statistics()
        assert statistics_before is not None

        redeemed_value_at_start = balance_str_to_int(statistics_before.redeemed_value)

        packets = [f"Ticket aggregation test: #{i:08d}" for i in range(ticket_count)]
        await send_and_receive_packets_with_pop(
            packets, src=swarm7[src], dest=swarm7[dest], path=[swarm7[mid].peer_id]
        )

        # monitor that the node aggregates and redeems tickets until the aggregated value is reached
        async def check_aggregate_and_redeem_tickets(api: HoprdAPI):
            while True:
                statistics_now = await api.get_tickets_statistics()
                assert statistics_now is not None

                redeemed_value_now = balance_str_to_int(statistics_now.redeemed_value)
                redeemed_value_diff = redeemed_value_now - redeemed_value_at_start

                # break out of the loop if the aggregated value is reached
                if redeemed_value_diff >= AGGREGATED_TICKET_PRICE:
                    break
                else:
                    await asyncio.sleep(0.1)

        await asyncio.wait_for(check_aggregate_and_redeem_tickets(swarm7[mid].api), 60.0)


# FIXME: This test depends on side-effects and cannot be run on its own. It
# should be redesigned.
@pytest.mark.asyncio
async def test_hoprd_sanity_check_channel_status(swarm7: dict[str, Node]):
    """
    The bash integration-test.sh opens and closes channels that can be visible inside this test scope
    """
    alice_api = swarm7["1"].api

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
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hoprd_check_native_withdraw(peer, swarm7: dict[str, Node]):
    amount = "9876"
    remaining_attempts = 10

    before_balance = int((await swarm7[peer].api.balances()).safe_native)
    await swarm7[peer].api.withdraw(amount, swarm7[peer].safe_address, "Native")

    after_balance = before_balance
    while remaining_attempts > 0:
        after_balance = int((await swarm7[peer].api.balances()).safe_native)
        if after_balance != before_balance:
            break
        await asyncio.sleep(0.5)
        remaining_attempts -= 1

    assert after_balance - before_balance == int(amount)


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hoprd_check_ticket_price_is_default(peer, swarm7: dict[str, Node]):
    price = await swarm7[peer].api.ticket_price()

    assert isinstance(price, int)
    assert price > 0

@pytest.mark.asyncio
@pytest.mark.parametrize("tag", [random.randint(0, RESERVED_TAG_UPPER_BOUND) for _ in range(5)])
async def test_send_message_with_reserved_application_tag_should_fail(tag: int, swarm7: dict[str, Node]):
    src, dest = random_distinct_pairs_from(barebone_nodes(), count=1)[0]

    assert await swarm7[src].api.send_message(
        swarm7[dest].peer_id, "This message should fail due to reserved tag", [], tag
    ) is None


@pytest.mark.asyncio
@pytest.mark.parametrize("tag", [random.randint(0, RESERVED_TAG_UPPER_BOUND) for _ in range(5)])
async def test_inbox_operations_with_reserved_application_tag_should_fail(tag: int, swarm7: dict[str, Node]):
    id = random.choice(barebone_nodes())

    assert await swarm7[id].api.messages_pop(tag) is None
    assert await swarm7[id].api.messages_peek(tag) is None
    assert await swarm7[id].api.messages_peek(tag) is None


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_peeking_messages_with_timestamp(src: str, dest: str, swarm7: dict[str, Node]):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)
    split_index = int(message_count * 0.66)

    random_tag = gen_random_tag()

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    packets = [f"0 hop message #{i:08d}" for i in range(message_count)]
    for packet in packets[:split_index]:
        await src_peer.api.send_message(dest_peer.peer_id, packet, [], random_tag)

    await asyncio.sleep(2)

    for packet in packets[split_index:]:
        await src_peer.api.send_message(dest_peer.peer_id, packet, [], random_tag)

    await asyncio.wait_for(
        check_received_packets_with_peek(dest_peer, packets, tag=random_tag, sort=True), MULTIHOP_MESSAGE_SEND_TIMEOUT
    )

    packets = await dest_peer.api.messages_peek_all(random_tag)
    timestamps = sorted([message.received_at for message in packets.messages])

    # ts_for_query set right before (1ms before) the first message of the second batch.
    # This is to ensure that the first message of the second batch will be returned by the query.
    # It's a workaround, it should work properly without the -1, however randmly fails.
    ts_for_query = timestamps[split_index] - 1

    async def peek_the_messages():
        packets = await dest_peer.api.messages_peek_all(random_tag, ts_for_query)

        assert len(packets.messages) == message_count - split_index

    await asyncio.wait_for(peek_the_messages(), MULTIHOP_MESSAGE_SEND_TIMEOUT)

    # Remove all messages so they do not interfere with the later tests
    await dest_peer.api.messages_pop_all(random_tag)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_send_message_return_timestamp(src: str, dest: str, swarm7: dict[str, Node]):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 10)
    random_tag = gen_random_tag()

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    packets = [f"0 hop message #{i:08d}" for i in range(message_count)]
    timestamps = []
    for packet in packets:
        res = await src_peer.api.send_message(dest_peer.peer_id, packet, [], random_tag)
        timestamps.append(res.timestamp)

    # Remove all messages so they do not interfere with the later tests
    await dest_peer.api.messages_pop_all(random_tag)

    assert len(timestamps) == message_count
    assert timestamps == sorted(timestamps)

