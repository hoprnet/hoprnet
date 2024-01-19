import asyncio
import os
import pytest
import random

from .conftest import TICKET_PRICE_PER_HOP, default_nodes
from .hopr import HoprdAPI
from .test_integration import create_channel, passive_node, send_and_receive_packets_with_pop


async def check_connected_peer_count(me, count):
    while True:
        if len([x["peer_id"] for x in await me.peers()]) >= count:
            break
        else:
            await asyncio.sleep(0.5)


@pytest.mark.asyncio
@pytest.mark.skipif(
    os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resource constraints"
)
@pytest.mark.parametrize("src,dest", [(random.choice(default_nodes()), passive_node())])
async def test_hoprd_stress_1_hop_to_self(src, dest, swarm7, cmd_line_args):
    STRESS_1_HOP_TO_SELF_MESSAGE_COUNT = cmd_line_args["stress_seq_request_count"]

    async with create_channel(
        swarm7[src],
        swarm7[dest],
        funding=STRESS_1_HOP_TO_SELF_MESSAGE_COUNT * TICKET_PRICE_PER_HOP,
        close_from_dest=False,
    ) as _channel_id:
        packets = [
            f"1 hop stress msg to self: {src} - {dest} - {src} #{i+1:08d}/{STRESS_1_HOP_TO_SELF_MESSAGE_COUNT:08d}"
            for i in range(STRESS_1_HOP_TO_SELF_MESSAGE_COUNT)
        ]

        await send_and_receive_packets_with_pop(
            packets, src=swarm7[src], dest=swarm7[src], path=[swarm7[dest]["peer_id"]], timeout=60.0
        )


@pytest.mark.asyncio
@pytest.mark.skipif(
    os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resource constraints"
)
async def test_stress_hoprd_send_message_should_send_sequential_messages_without_errors(request, cmd_line_args):
    if "127.0.0.1" in cmd_line_args["stress_tested_api"] or "localhost" in cmd_line_args["stress_tested_api"]:
        request.getfixturevalue("swarm7")

    STRESS_SEQUENTIAL_MESSAGE_COUNT = cmd_line_args["stress_seq_request_count"]

    stressor = HoprdAPI(cmd_line_args["stress_tested_api"], cmd_line_args["stress_tested_api_token"])
    await asyncio.wait_for(check_connected_peer_count(stressor, count=cmd_line_args["stress_minimum_peer_count"]), 20.0)

    connected_peers = [x["peer_id"] for x in await stressor.peers()]

    assert len(connected_peers) >= cmd_line_args["stress_minimum_peer_count"]

    assert all(
        [
            (await stressor.send_message(random.choice(connected_peers), f"message #{i}", []))
            for i in range(STRESS_SEQUENTIAL_MESSAGE_COUNT)
        ]
    )


@pytest.mark.asyncio
@pytest.mark.skipif(os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resources")
async def test_stress_hoprd_send_message_should_send_parallel_messages_without_errors(request, cmd_line_args):
    if "127.0.0.1" in cmd_line_args["stress_tested_api"] or "localhost" in cmd_line_args["stress_tested_api"]:
        request.getfixturevalue("swarm7")

    STRESS_PARALLEL_MESSAGE_COUNT = cmd_line_args["stress_seq_request_count"]

    stressor = HoprdAPI(cmd_line_args["stress_tested_api"], cmd_line_args["stress_tested_api_token"])
    await asyncio.wait_for(check_connected_peer_count(stressor, count=cmd_line_args["stress_minimum_peer_count"]), 20.0)

    connected_peers = [x["peer_id"] for x in await stressor.peers()]

    assert len(connected_peers) >= cmd_line_args["stress_minimum_peer_count"]

    assert all(
        await asyncio.gather(
            *[
                stressor.send_message(random.choice(connected_peers), f"message #{i}", [])
                for i in range(STRESS_PARALLEL_MESSAGE_COUNT)
            ]
        )
    )
