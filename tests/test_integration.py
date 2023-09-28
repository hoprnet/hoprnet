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
    TICKET_PRICE_PER_HOP
)


PARAMETERIZED_SAMPLE_SIZE = 1 if os.getenv('CI', default="false") == "false" else 3
AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP
MULTIHOP_MESSAGE_SEND_TIMEOUT = 10.0        #s

def shuffled(coll):
    random.shuffle(coll)
    return coll


@asynccontextmanager
async def create_channel(src, dest, funding):
    channel = await src['api'].open_channel(dest['address'], funding)
    assert channel is not None
    try:
        yield channel
    finally:
        assert await src['api'].close_channel(channel)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest",
    random.sample([(src,dest) for src, dest in itertools.product(
        list(NODES.keys())[:5], repeat=2) if src != dest], PARAMETERIZED_SAMPLE_SIZE)
)
async def test_hoprd_ping_should_work_between_nodes_in_the_same_network(src, dest, setup_7_nodes):
    pinger = setup_7_nodes[src]['api']
    
    response = await pinger.ping(setup_7_nodes[dest]['peer_id'])
    
    assert response is not None
    assert int(response.latency) > 0, f"PNon-0 round trip time expected, actual: '{int(response.latency)}'"


@pytest.mark.asyncio
async def test_hoprd_ping_should_timeout_on_pinging_self(setup_7_nodes):
    pinger = setup_7_nodes["Alice"]['api']
    
    response = await pinger.ping(setup_7_nodes["Alice"]['peer_id'])
    
    assert response is None, f"Pinging self should produce timeout, not '{response}'"


@pytest.mark.asyncio
@pytest.mark.parametrize("node", list(NODES.keys())[:5])
async def test_hoprd_should_not_have_unredeemed_tickets_without_sending_messages(node, setup_7_nodes):
    """
    log "Node 2 has no unredeemed ticket value"
    result=$(api_get_ticket_statistics "${api2}" "\"unredeemedValue\":\"0\"")
    log "-- ${result}"
    """
    statistics = await setup_7_nodes[node]['api'].get_tickets_statistics()
    assert int(statistics.unredeemed_value) == 0
    assert int(statistics.unredeemed) == 0


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest",
    random.sample([(src,dest) for src, dest in itertools.product(
        list(NODES.keys())[:5], repeat=2) if src != dest], PARAMETERIZED_SAMPLE_SIZE)
)
async def test_hoprd_should_be_able_to_send_0_hop_messages_without_open_channels(src, dest, setup_7_nodes):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 5)
    
    packets = [f"0 hop message #{i:04d}" for i in range(message_count)]
    
    for packet in packets:
        assert await setup_7_nodes[src]['api'].send_message(setup_7_nodes[dest]['peer_id'], packet, [])

    await asyncio.sleep(1)
    
    async def check_received():
        received = [(await setup_7_nodes[dest]['api'].messages_pop()).body for i in range(len(packets))]
        received.sort()
        assert received == packets
            
    await asyncio.wait_for(check_received(), MULTIHOP_MESSAGE_SEND_TIMEOUT)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest",
    random.sample([(src,dest) for src, dest in itertools.product(
        list(NODES.keys())[:5], repeat=2) if src != dest], PARAMETERIZED_SAMPLE_SIZE)
)
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_1_hop_to_self_scenario(src, dest, setup_7_nodes):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 5)
    
    assert(setup_7_nodes[dest]['peer_id'] in [x['peer_id'] for x in await setup_7_nodes[src]['api'].peers()])
    assert(setup_7_nodes[src]['peer_id'] in [x['peer_id'] for x in await setup_7_nodes[dest]['api'].peers()])

    async with create_channel(setup_7_nodes[src],
                              setup_7_nodes[dest],
                              funding=str(message_count * TICKET_PRICE_PER_HOP)) as channel:
        await asyncio.sleep(3)
        
        packets = [f"1 hop message to self #{i:04d}" for i in range(message_count)]
        
        for packet in packets:
            assert await setup_7_nodes[src]['api'].send_message(
                setup_7_nodes[src]['peer_id'], packet, [setup_7_nodes[dest]['peer_id']])

        await asyncio.sleep(1)
        
        async def check_received():
            received = [(await setup_7_nodes[src]['api'].messages_pop()).body for i in range(len(packets))]
            received.sort()
            assert received == packets

        await asyncio.wait_for(check_received(), MULTIHOP_MESSAGE_SEND_TIMEOUT)
        
        statistics = await setup_7_nodes[dest]['api'].get_tickets_statistics()
        assert (statistics.redeemed + statistics.unredeemed) > 0
        
        assert await setup_7_nodes[dest]['api'].channel_redeem_tickets(channel)
        
        async def channel_redeemed():
            while (await setup_7_nodes[dest]['api'].get_tickets_statistics()).unredeemed > 0:
                await asyncio.sleep(0.5)
        
        await asyncio.wait_for(channel_redeemed(), 30.0)


@pytest.mark.asyncio
@pytest.mark.parametrize("route",
    [shuffled(list(NODES.keys()))[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)] +
    [shuffled(list(NODES.keys()))[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
)
async def test_hoprd_should_create_redeemable_tickets_on_routing_in_general_n_hop(route, setup_7_nodes):
    message_count = int(TICKET_AGGREGATION_THRESHOLD / 2)
    
    assert all([
        setup_7_nodes[route[i+1]]['peer_id'] in [x['peer_id'] for x in await setup_7_nodes[route[i]]['api'].peers()]
        for i in range(len(route) - 1)
    ])
    
    async with AsyncExitStack() as channels:
        await asyncio.gather(*[
            channels.enter_async_context(
                create_channel(setup_7_nodes[route[i]],
                               setup_7_nodes[route[i+1]],
                               funding=str(message_count * TICKET_PRICE_PER_HOP))
                ) for i in range(len(route) - 1)
        ])
        
        await asyncio.sleep(1)
        
        packets = [f"hoppity message #{i:04d}" for i in range(message_count)]
        
        for packet in packets:
            assert await setup_7_nodes[route[0]]['api'].send_message(
                setup_7_nodes[route[-1]]['peer_id'],
                packet,
                [setup_7_nodes[x]['peer_id'] for x in route[1:-1]])

        await asyncio.sleep(2)
        
        async def check_received():
            received = [(await setup_7_nodes[route[-1]]['api'].messages_pop()).body for i in range(len(packets))]
            received.sort()
            assert received == packets

        await asyncio.wait_for(check_received(), MULTIHOP_MESSAGE_SEND_TIMEOUT)
        
        statistics = await setup_7_nodes[route[1]]['api'].get_tickets_statistics()
        assert (statistics.redeemed + statistics.unredeemed) > 0
        
        assert await setup_7_nodes[route[1]]['api'].tickets_redeem()
        
        async def all_redeemed():
            while (await setup_7_nodes[route[1]]['api'].get_tickets_statistics()).unredeemed > 0:
                await asyncio.sleep(0.5)
        
        await asyncio.wait_for(all_redeemed(), 30.0)


@pytest.mark.asyncio
async def test_hoprd_ping_should_not_be_able_to_ping_nodes_in_other_network_UNFINISHED(setup_7_nodes): 
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
async def test_hoprd_ping_should_not_be_able_to_ping_nodes_not_present_in_the_registry_UNFINISHED(setup_7_nodes): 
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
@pytest.mark.parametrize("route",
    [shuffled(list(NODES.keys()))[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)]
)
async def test_hoprd_strategy_automatic_ticket_aggregation_and_redeeming(route, setup_7_nodes):
    ticket_count = TICKET_AGGREGATION_THRESHOLD*2

    assert all([
        setup_7_nodes[route[i+1]]['peer_id'] in [x['peer_id'] for x in await setup_7_nodes[route[i]]['api'].peers()]
        for i in range(len(route) - 1)
    ])
    
    async with AsyncExitStack() as channels:
        await asyncio.gather(*[
            channels.enter_async_context(
                create_channel(setup_7_nodes[route[i]],
                               setup_7_nodes[route[i+1]],
                               funding=str(ticket_count * TICKET_PRICE_PER_HOP)
                )) for i in range(len(route) - 1)
        ])

        await asyncio.sleep(3)

        statistics_before = await setup_7_nodes[route[1]]['api'].get_tickets_statistics()

        for i in range(ticket_count):
            assert await setup_7_nodes[route[0]]['api'].send_message(
                setup_7_nodes[route[-1]]['peer_id'],
                f"#{i}",
                [setup_7_nodes[route[1]]['peer_id']])

        await asyncio.sleep(1)

        for i in range(ticket_count):
            await setup_7_nodes[route[-1]]['api'].messages_pop()

        async def aggregate_and_redeem_tickets():
            while True:
                statistics_after = await setup_7_nodes[route[1]]['api'].get_tickets_statistics()
                redeemed_value = int(statistics_after.redeemed_value) - int(statistics_before.redeemed_value)
                redeemed_ticket_count = statistics_after.redeemed - statistics_before.redeemed

                if redeemed_value >= AGGREGATED_TICKET_PRICE:
                    break
                else:
                    await asyncio.sleep(0.5)

            assert(redeemed_value >= AGGREGATED_TICKET_PRICE)
            assert(redeemed_ticket_count == pytest.approx(redeemed_value / AGGREGATED_TICKET_PRICE, 0.1))
        
        await asyncio.wait_for(aggregate_and_redeem_tickets(), 60.0)


def test_hoprd_protocol_bash_integration_tests(setup_7_nodes):
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

    nodes_api_as_str = " ".join(list(map(lambda x: f"\"localhost:{x['api_port']}\"", setup_7_nodes.values())))

    log_file_path = f"/tmp/hopr-smoke-{__name__}.log"
    subprocess.run(
        ['bash', '-o', 'pipefail', '-c', f"./tests/integration-test.sh {nodes_api_as_str} 2>&1 | tee {log_file_path}"],
        shell=False,
        capture_output=True,
        env=env_vars,
        # timeout=2000,
        check=True,
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("source,dest",
    [("Alice", "Dave"), ("Alice", "Bob"), ("Bob", "Camilla"), ("Camilla", "Dave"), ("Dave", "Eva"), ("Eva", "Alice")]
)
async def test_hoprd_should_be_able_to_close_open_channels_with_possible_tickets(setup_7_nodes, source, dest):
    src = setup_7_nodes[source]["api"]
    open_channels = await src.all_channels(include_closed=False)
    channels = [
        oc.channel_id for oc in open_channels.all \
            if oc.source_address == setup_7_nodes[source]['address'] \
                and oc.destination_address == setup_7_nodes[dest]['address']
        ]

    assert len(channels) == 1, f"There should be exactly one channel open from {source} to {dest}"
    assert await src.close_channel(channels[0])


@pytest.mark.asyncio
async def test_hoprd_should_be_able_to_open_and_close_channel_without_tickets(setup_7_nodes):
    alice = setup_7_nodes['Alice']
    eva = setup_7_nodes['Eva']

    alice_api = alice['api']

    open_channels = await alice_api.all_channels(include_closed=False)
    channel_ids = [
        oc.channel_id for oc in open_channels.all \
            if oc.source_address == alice['address'] and oc.destination_address == eva['address']
    ]
    assert len(channel_ids) == 0, "No channel from 'Alice' to 'Eva' should exist at this point"

    assert await alice_api.open_channel(eva['address'], OPEN_CHANNEL_FUNDING_VALUE), "Channel should be opened"

    await asyncio.sleep(3)

    open_channels = await alice_api.all_channels(include_closed=False)
    channel_ids = [
        oc.channel_id for oc in open_channels.all \
            if oc.source_address == alice['address'] and oc.destination_address == eva['address']
        ]
    assert len(channel_ids) == 1, "There should be a channel from 'Alice' to 'Eva' at this point"

    assert await alice_api.close_channel(channel_ids[0])


@pytest.mark.asyncio
async def test_hoprd_sanity_check_channel_status(setup_7_nodes):
    alice_api = setup_7_nodes['Alice']['api']

    open_channels = await alice_api.all_channels(include_closed=False)
    open_and_closed_channels = await alice_api.all_channels(include_closed=True)

    assert len(open_and_closed_channels.all) >= len(open_channels.all), "Open and closed channels should be present"

    statuses = [c.status for c in open_and_closed_channels.all]
    assert 'Closed' in statuses or 'PendingToClose' in statuses, "Closed channels should be present"


@pytest.mark.asyncio
async def test_hoprd_strategy_UNFINISHED():
    """
    # NOTE: strategy testing will require separate setup so commented out for now until moved
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
