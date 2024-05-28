import asyncio
from contextlib import AsyncExitStack
import os
import random

import pytest

from .conftest import TICKET_PRICE_PER_HOP, to_ws_url
from .hopr import HoprdAPI
from .test_integration import create_channel, send_and_receive_packets_with_pop

import json
import random
import re

import websockets

from .conftest import API_TOKEN, nodes_with_auth, random_distinct_pairs_from
from .node import Node


@pytest.fixture
async def stress_fixture(request: pytest.FixtureRequest):
    return {
        "request_count": request.config.getoption("--stress-request-count"),
        "sources": json.loads(request.config.getoption("--stress-sources")),
        "target": json.loads(request.config.getoption("--stress-target")),
    }


async def has_peer(me, target):
    while True:
        if target in [x["peer_id"] for x in await me.peers()]:
            import logging

            logging.error("peer {} not found in {await me.peers()}".format(["peer_id"]))
            break
        else:
            await asyncio.sleep(1)
            continue


@pytest.mark.asyncio
@pytest.mark.skipif(
    os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resource constraints"
)
async def test_stress_relayed_flood_test_with_sources_performing_1_hop_to_self(stress_fixture, swarm7):
    STRESS_1_HOP_TO_SELF_MESSAGE_COUNT = stress_fixture["request_count"]

    api_sources = [HoprdAPI(f'http://{d["url"]}', d["token"]) for d in stress_fixture["sources"]]
    api_target = HoprdAPI(f'http://{stress_fixture["target"]["url"]}', stress_fixture["target"]["token"])
    target_peer_id = await api_target.addresses("hopr")

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[asyncio.wait_for(has_peer(source, target_peer_id), timeout=15.0) for source in api_sources]
        )

        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(
                        source,
                        api_target,
                        funding=STRESS_1_HOP_TO_SELF_MESSAGE_COUNT * TICKET_PRICE_PER_HOP,
                        close_from_dest=False,
                    )
                )
                for source in api_sources
            ]
        )

        async def send_and_receive_all_messages(host, port, token, target_peer_id):
            socket = websockets.connect(
                f"{to_ws_url(host, port)}",
                header={"X-Auth-Token": token},
            )

            tag = random.randint(30000, 60000)
            packets = [
                f"1 hop stress msg to self ({host}:{port}) through {target_peer_id} #{i+1:08d}/{STRESS_1_HOP_TO_SELF_MESSAGE_COUNT:08d}"
                for i in range(STRESS_1_HOP_TO_SELF_MESSAGE_COUNT)
            ]

            recv_packets = []

            for packet in packets:
                msg = {"cmd": "sendmsg", "args": {"body": packet, "peerId": target_peer_id, "path": [], "tag": tag}}
                await socket.send(json.dumps(msg))

            # receive all messages, acks and ack-challenges
            for _ in range(packets.len() * 3):
                try:
                    msg = await asyncio.wait_for(socket.recv(), timeout=5)
                    if re.match(r"^.*\"message\".*$", msg):
                        recv_packets.append(msg)
                except Exception:
                    pytest.fail(f"Timeout when receiving an expected item from socket")

            packets.sort()
            recv_packets.sort()
            assert packets == recv_packets

        await asyncio.gather(
            *[
                asyncio.wait_for(
                    send_and_receive_all_messages(
                        source["url"].split(":")[0], source["url"].split(":")[1], source["token"], target_peer_id
                    ),
                    timeout=20.0,
                )
                for source in api_sources
            ]
        )
