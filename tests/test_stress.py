import asyncio
import json
import logging
import os
import time
from contextlib import AsyncExitStack

import pytest
from api_lib.headers.authorization import Bearer
from websockets.asyncio.client import connect

from sdk.python.api.hopr import HoprdAPI
from sdk.python.localcluster.constants import TICKET_PRICE_PER_HOP

from .conftest import to_ws_url
from .test_integration import create_channel

logging.basicConfig(format="%(asctime)s %(message)s")


@pytest.fixture
async def stress_fixture(request: pytest.FixtureRequest):
    return {
        "request_count": request.config.getoption("--stress-request-count"),
        "sources": json.loads(request.config.getoption("--stress-sources")),
        "target": json.loads(request.config.getoption("--stress-target")),
    }


async def peer_is_present(me: HoprdAPI, target):
    while True:
        if target in [x.address for x in await me.peers()]:
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
    ROUGH_PAYLOAD_SIZE = 460

    api_sources = [HoprdAPI(f"http://{d['url']}", Bearer(d["token"], "/api/v4")) for d in stress_fixture["sources"]]
    api_target = HoprdAPI(
        f"http://{stress_fixture['target']['url']}", Bearer(stress_fixture["target"]["token"], "/api/v4")
    )
    target_address = (await api_target.addresses()).native

    async with AsyncExitStack() as channels:
        await asyncio.gather(
            *[asyncio.wait_for(peer_is_present(source, target_address), timeout=15.0) for source in api_sources]
        )

        await asyncio.gather(
            *[
                channels.enter_async_context(
                    create_channel(
                        ApiWrapper(source, (await source.addresses()).native),
                        ApiWrapper(api_target, (await api_target.addresses()).native),
                        funding=STRESS_1_HOP_TO_SELF_MESSAGE_COUNT * TICKET_PRICE_PER_HOP * 3,
                        close_from_dest=False,
                    )
                )
                for source in api_sources
            ],
            *[
                channels.enter_async_context(
                    create_channel(
                        ApiWrapper(api_target, (await api_target.addresses()).native),
                        ApiWrapper(source, (await source.addresses()).native),
                        funding=STRESS_1_HOP_TO_SELF_MESSAGE_COUNT * TICKET_PRICE_PER_HOP * 3,
                        close_from_dest=False,
                    )
                )
                for source in api_sources
            ],
        )

        async def send_and_receive_all_messages(host, port, token, self_address):
            event = asyncio.Event()

            data = bytearray(os.urandom(ROUGH_PAYLOAD_SIZE))

            async def compare_data(reader, writer):
                read = bytearray()
                start_time = time.time()

                while len(read) < len(data):
                    read += await reader.read(min(1024, len(data) - len(read)))

                end_time = time.time()

                logging.info(
                    "The websocket stress test ran at "
                    + f"{STRESS_1_HOP_TO_SELF_MESSAGE_COUNT / (end_time - start_time)} packets/s/node"
                )

                assert read == data, f"Received data does not match the sent data: {read} != {data}"

                event.set()

            server = await asyncio.start_server(compare_data, "127.0.0.1", 0)
            assigned_port = server.sockets[0].getsockname()[1]

            async with server:
                async with connect(
                    to_ws_url(
                        host,
                        port,
                        args=[
                            ("target", f"127.0.0.1:{assigned_port}"),
                            ("hops", 1),
                            ("capabilities", "Segmentation"),
                            ("capabilities", "Retransmission"),
                            ("destination", f"{self_address}"),
                        ],
                    ),
                    additional_headers=[("X-Auth-Token", token)],
                ) as ws:
                    await ws.send(data)
                    await asyncio.wait_for(event.wait(), timeout=60.0)

        await asyncio.gather(
            *[
                asyncio.wait_for(
                    send_and_receive_all_messages(
                        source["url"].split(":")[0],
                        source["url"].split(":")[1],
                        source["token"],
                        (
                            await HoprdAPI(f"http://{source['url']}", Bearer(source["token"], "/api/v4")).addresses()
                        ).native,
                    ),
                    timeout=60.0,
                )
                for source in stress_fixture["sources"]
            ]
        )
