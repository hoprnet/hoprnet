import json
import os
import subprocess
import pytest

import asyncio
from conftest import DEFAULT_API_TOKEN, OPEN_CHANNEL_FUNDING_VALUE, TICKET_AGGREGATION_THRESHOLD, TICKET_PRICE_PER_HOP

from hopr import HoprdAPI


AGGREGATED_TICKET_PRICE = TICKET_AGGREGATION_THRESHOLD * TICKET_PRICE_PER_HOP


def test_hoprd_protocol_integration_tests(setup_7_nodes):
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
async def test_hoprd_protocol_aggregated_ticket_redeeming(setup_7_nodes):
    alice = setup_7_nodes['Alice']
    bob = setup_7_nodes['Bob']
    camilla = setup_7_nodes['Camilla']
        
    alice_api = HoprdAPI(f"http://localhost:{alice['api_port']}", DEFAULT_API_TOKEN)
    bob_api = HoprdAPI(f"http://localhost:{bob['api_port']}", DEFAULT_API_TOKEN)
    camilla_api = HoprdAPI(f"http://localhost:{camilla['api_port']}", DEFAULT_API_TOKEN)
    
    assert(bob['peer_id'] in [x['peer_id'] for x in await alice_api.peers()])
    assert(camilla['peer_id'] in [x['peer_id'] for x in await bob_api.peers()])
    
    assert await alice_api.open_channel(bob['address'], OPEN_CHANNEL_FUNDING_VALUE)
    assert await bob_api.open_channel(camilla['address'], OPEN_CHANNEL_FUNDING_VALUE)
    
    statistics_before = await bob_api.get_tickets_statistics()
    
    for i in range(TICKET_AGGREGATION_THRESHOLD):
        r = await alice_api.send_message(camilla['peer_id'], f"#{i}", [bob['peer_id']]), "Failed to send 1 hop message"
        assert r
        
    await asyncio.sleep(1)
        
    for i in range(TICKET_AGGREGATION_THRESHOLD):
        await camilla_api.messages_pop()
    
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
