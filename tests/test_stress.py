import asyncio
from contextlib import AsyncExitStack
import json
import os
import random
import time

import logging
import pytest
import websockets

from .conftest import TICKET_PRICE_PER_HOP, to_ws_url
from .hopr import HoprdAPI
from .test_integration import create_channel


logging.basicConfig(format="%(asctime)s %(message)s")


@pytest.fixture
async def stress_fixture(request: pytest.FixtureRequest):
    return {
        "request_count": request.config.getoption("--stress-request-count"),
        "sources": json.loads(request.config.getoption("--stress-sources")),
        "target": json.loads(request.config.getoption("--stress-target")),
    }


async def peer_is_present(me, target):
    while True:
        if target in [x["peer_id"] for x in await me.peers()]:
            break
        else:
            await asyncio.sleep(1)
            continue


class ApiWrapper:
    def __init__(self, api, address):
        self.inner = api
        self.addr = address

    @property
    def api(self):
        return self.inner

    @property
    def address(self):
        return self.addr


@pytest.mark.asyncio
@pytest.mark.skipif(
    os.getenv("CI", "false") == "true", reason="stress tests fail randomly on CI due to resource constraints"
)
async def test_stress_relayed_flood_test_with_sources_performing_1_hop_to_self(stress_fixture):
    STRESS_1_HOP_TO_SELF_MESSAGE_COUNT = stress_fixture["request_count"]

    api_sources = [HoprdAPI(f'http://{d["url"]}', d["token"]) for d in stress_fixture["sources"]]
    api_target = HoprdAPI(f'http://{stress_fixture["target"]["url"]}', stress_fixture["target"]["token"])
    target_peer_id = await api_target.addresses("hopr")

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[asyncio.wait_for(peer_is_present(source, target_peer_id), timeout=15.0) for source in api_sources]
        )

        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(
                        ApiWrapper(source, await source.addresses("native")),
                        ApiWrapper(api_target, await api_target.addresses("native")),
                        funding=STRESS_1_HOP_TO_SELF_MESSAGE_COUNT * TICKET_PRICE_PER_HOP,
                        close_from_dest=False,
                    )
                )
                for source in api_sources
            ]
        )

        async def send_and_receive_all_messages(host, port, token, self_peer_id, target_peer_id):
            start_time = time.time()

            async with websockets.connect(
                f"{to_ws_url(host, port)}",
                extra_headers=[("X-Auth-Token", token)],
            ) as socket:
                tag = random.randint(30000, 60000)
                packets = [
                    f"1 hop stress msg to self ({host}:{port}) through {target_peer_id} #{i+1:08d}/{STRESS_1_HOP_TO_SELF_MESSAGE_COUNT:08d}"
                    for i in range(STRESS_1_HOP_TO_SELF_MESSAGE_COUNT)
                ]

                recv_packets = []
                recv_ack_challenges = 0
                recv_acks = 0

                for packet in packets:
                    msg = {
                        "cmd": "sendmsg",
                        "args": {"body": packet, "peerId": self_peer_id, "path": [target_peer_id], "tag": tag},
                    }
                    await socket.send(json.dumps(msg))

                packets.sort()

                # receive all messages, acks and ack-challenges
                for _ in range(len(packets) * 3):
                    try:
                        msg = await asyncio.wait_for(socket.recv(), timeout=5)
                        msg = json.loads(msg)
                        if msg["type"] == "message-ack":
                            recv_acks += 1
                        elif msg["type"] == "message-ack-challenge":
                            recv_ack_challenges += 1
                        elif msg["type"] == "message":
                            recv_packets.append(msg["body"])
                    except Exception:
                        break

                end_time = time.time()

                logging.info(
                    f"The websocket stress test ran at {STRESS_1_HOP_TO_SELF_MESSAGE_COUNT/(end_time - start_time)} packets/s/node"
                )

                recv_packets.sort()
                assert recv_packets == packets
                assert recv_acks == len(packets)
                assert recv_ack_challenges == len(packets)

        await asyncio.gather(
            *[
                asyncio.wait_for(
                    send_and_receive_all_messages(
                        source["url"].split(":")[0],
                        source["url"].split(":")[1],
                        source["token"],
                        await HoprdAPI(f'http://{source["url"]}', source["token"]).addresses("hopr"),
                        target_peer_id,
                    ),
                    timeout=60.0,
                )
                for source in stress_fixture["sources"]
            ]
        )
