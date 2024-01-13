from tests.conftest import DEFAULT_API_TOKEN
from contextlib import asynccontextmanager
import random
import websockets
import json
import pytest
import asyncio
import re
import rlp


LOCALHOST = "127.0.0.1"


def url(port):
    return f"ws://{LOCALHOST}:{port}/api/v3/messages/websocket"


EXTRA_HEADERS = [("Cookie", f"X-Auth-Token={DEFAULT_API_TOKEN}")]


async def connect_to(port):
    return await websockets.connect(url(port), extra_headers=EXTRA_HEADERS)


@asynccontextmanager
async def web_socket_connection(port):
    conn = await connect_to(port)
    try:
        yield conn
    finally:
        await conn.close()


@pytest.fixture
async def ws_connections(swarm7):
    async with web_socket_connection(swarm7["1"]["api_port"]) as ws1, web_socket_connection(
        swarm7["2"]["api_port"]
    ) as ws2, web_socket_connection(swarm7["3"]["api_port"]) as ws3, web_socket_connection(
        swarm7["4"]["api_port"]
    ) as ws4, web_socket_connection(
        swarm7["5"]["api_port"]
    ) as ws5:
        yield {"1": ws1, "2": ws2, "3": ws3, "4": ws4, "5": ws5}


async def test_websocket_send_receive_messages(swarm7, ws_connections):
    # FIXME: for some reason sending NAT-to-NAT does not work in the test setup
    # valid_node_keys = ["1", "2", "3", "4", "5"]
    valid_node_keys = ["1", "2", "3", "4"]
    message_target_count = 50

    for i in range(message_target_count):
        [source_key, target_key] = random.sample(valid_node_keys, 2)

        source_peer = swarm7[source_key]["peer_id"]
        target_peer = swarm7[target_key]["peer_id"]

        source_ws = ws_connections[source_key]
        target_ws = ws_connections[target_key]

        body = f"hello msg {i} from peer {source_peer} to peer {target_peer}"

        # we test direct messaging only
        msg = {"cmd": "sendmsg", "args": {"body": body, "recipient": target_peer, "path": []}}

        await source_ws.send(json.dumps(msg))

        try:
            ack_challenge_msg = await asyncio.wait_for(source_ws.recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving ack-challenge of msg {i} from {source_key} to {target_key}")
        assert re.match(r"^ack-challenge:.*$", ack_challenge_msg)

        try:
            inc_msg_rlp = await asyncio.wait_for(target_ws.recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving msg {i} from {source_key} to {target_key}")
        [inc_msg_str, _] = rlp.decode(bytearray([int(c) for c in inc_msg_rlp.split(",")]))
        assert inc_msg_str.decode("unicode_escape") == body

        try:
            ack_msg = await asyncio.wait_for(source_ws.recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving ack of msg {i} from {source_key} to {target_key}")
        assert re.match(r"^ack:.*$", ack_msg)
