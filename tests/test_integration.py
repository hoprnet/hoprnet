import json
import os
import subprocess
import pytest

import asyncio
from conftest import DEFAULT_API_TOKEN, OPEN_CHANNEL_FUNDING_VALUE, TICKET_AGGREGATION_THRESHOLD, TICKET_PRICE_PER_HOP


AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP

@pytest.mark.asyncio
async def test_hoprd_protocol_aggregated_ticket_redeeming(setup_7_nodes):
    alice = setup_7_nodes['Alice']
    bob = setup_7_nodes['Bob']
    camilla = setup_7_nodes['Camilla']

    alice_api = alice['api']
    bob_api = bob['api']
    camilla_api = camilla['api']

    assert(bob['peer_id'] in [x['peer_id'] for x in await alice_api.peers()])
    assert(camilla['peer_id'] in [x['peer_id'] for x in await bob_api.peers()])

    assert await alice_api.open_channel(bob['address'], OPEN_CHANNEL_FUNDING_VALUE)
    assert await bob_api.open_channel(camilla['address'], OPEN_CHANNEL_FUNDING_VALUE)

    await asyncio.sleep(3)

    statistics_before = await bob_api.get_tickets_statistics()

    for i in range(TICKET_AGGREGATION_THRESHOLD*2):
        assert await alice_api.send_message(camilla['peer_id'], f"#{i}", [bob['peer_id']])

    await asyncio.sleep(1)

    for i in range(TICKET_AGGREGATION_THRESHOLD*2):
        assert await camilla_api.messages_pop() is not None

    # wait for tickets to be aggregated and redeemed
    for _ in range(60):
        statistics_after = await bob_api.get_tickets_statistics()
        redeemed_value = int(statistics_after.redeemed_value) - int(statistics_before.redeemed_value)
        redeemed_ticket_count = statistics_after.redeemed - statistics_before.redeemed

        if redeemed_value >= AGGREGATED_TICKET_PRICE:
            break
        else:
            await asyncio.sleep(0.5)

    assert(redeemed_value >= AGGREGATED_TICKET_PRICE)
    assert(redeemed_ticket_count == pytest.approx(redeemed_value / AGGREGATED_TICKET_PRICE, 0.1))


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
async def test_hoprd_should_be_able_to_redeem_all_tickets_at_this_point_UNFINISHED(setup_7_nodes):
    assert True


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
