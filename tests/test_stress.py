import asyncio
import logging
import os
import pytest
import random

from hoprd.wrapper import HoprdAPI

HTTP_STATUS_CODE_OK = 200
HTTP_STATUS_CODE_SEND_MESSAGE_OK = 202


@pytest.mark.asyncio
@pytest.mark.skipif(os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resources")
async def test_stress_hoprd_send_message_should_send_sequential_messages_without_errors(request, cmd_line_args):
    if "127.0.0.1" in cmd_line_args["stress_tested_api"] or "localhost" in cmd_line_args["stress_tested_api"]:
        request.getfixturevalue("setup_7_nodes")

    STRESS_SEQUENTIAL_MESSAGE_COUNT = cmd_line_args["stress_seq_request_count"]

    alice = HoprdAPI(cmd_line_args["stress_tested_api"], cmd_line_args["stress_tested_api_token"])
    hops = []  # zero hop

    # wait for peers to be connected
    for _ in range(60):
        if (
            len([i["peerId"] for i in (await alice.peers()).json()["connected"]])
            >= cmd_line_args["stress_minimum_peer_count"]
        ):
            logging.info(f'peers ready {[i["peerId"] for i in (await alice.peers()).json()["connected"]]}')
            break
        else:
            await asyncio.sleep(1)

    connected_peers = [i["peerId"] for i in (await alice.peers()).json()["connected"]]
    assert len(connected_peers) >= cmd_line_args["stress_minimum_peer_count"]

    expected = [HTTP_STATUS_CODE_SEND_MESSAGE_OK] * STRESS_SEQUENTIAL_MESSAGE_COUNT
    actual = [
        (await alice.send_message(random.choice(connected_peers), f"message #{i}", hops)).status_code
        for i in range(STRESS_SEQUENTIAL_MESSAGE_COUNT)
    ]
    assert expected == actual


@pytest.mark.asyncio
@pytest.mark.skipif(os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resources")
async def test_stress_hoprd_send_message_should_send_parallel_messages_without_errors(request, cmd_line_args):
    if "127.0.0.1" in cmd_line_args["stress_tested_api"] or "localhost" in cmd_line_args["stress_tested_api"]:
        request.getfixturevalue("setup_7_nodes")

    STRESS_PARALLEL_MESSAGE_COUNT = cmd_line_args["stress_par_request_count"]

    alice = HoprdAPI(cmd_line_args["stress_tested_api"], cmd_line_args["stress_tested_api_token"])
    hops = []  # zero hop

    # wait for peers to be connected
    for _ in range(60):
        if (
            len([i["peerId"] for i in (await alice.peers()).json()["connected"]])
            >= cmd_line_args["stress_minimum_peer_count"]
        ):
            logging.info(f'peers ready {[i["peerId"] for i in (await alice.peers()).json()["connected"]]}')
            break
        else:
            await asyncio.sleep(1)

    connected_peers = [i["peerId"] for i in (await alice.peers()).json()["connected"]]
    assert len(connected_peers) >= cmd_line_args["stress_minimum_peer_count"]

    expected = [HTTP_STATUS_CODE_SEND_MESSAGE_OK] * STRESS_PARALLEL_MESSAGE_COUNT
    actual = await asyncio.gather(
        *[
            alice.send_message(random.choice(connected_peers), f"message #{i}", hops)
            for i in range(STRESS_PARALLEL_MESSAGE_COUNT)
        ]
    )
    actual = [i.status_code for i in actual]

    assert expected == actual
