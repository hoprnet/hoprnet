from tests.conftest import DEFAULT_API_TOKEN
import random
import websockets
import json
import pytest
import asyncio
import re
import rlp

LOCALHOST = "127.0.0.1"


def url(port):
    return f"ws://{LOCALHOST}:{port}/api/v2/messages/websocket"


def cookie():
    return f"X-Auth-Token={DEFAULT_API_TOKEN}"


@pytest.fixture
async def ws_connections(setup_7_nodes):
    ws1 = await websockets.connect(url(setup_7_nodes["1"]["api_port"]), extra_headers=[("Cookie", cookie())])
    ws2 = await websockets.connect(url(setup_7_nodes["2"]["api_port"]), extra_headers=[("Cookie", cookie())])
    ws3 = await websockets.connect(url(setup_7_nodes["3"]["api_port"]), extra_headers=[("Cookie", cookie())])
    ws4 = await websockets.connect(url(setup_7_nodes["4"]["api_port"]), extra_headers=[("Cookie", cookie())])
    ws5 = await websockets.connect(url(setup_7_nodes["5"]["api_port"]), extra_headers=[("Cookie", cookie())])
    yield {"1": ws1, "2": ws2, "3": ws3, "4": ws4, "5": ws5}
    await ws1.close()
    await ws2.close()
    await ws3.close()
    await ws4.close()
    await ws5.close()


@pytest.fixture
async def ws2(setup_7_nodes):
    ws = await websockets.connect(url(setup_7_nodes["4"]["api_port"]), extra_headers=[("Cookie", cookie())])
    yield ws
    await ws.close()


# This test sends 50 messages via websocket between 2 randomly chosen peers
# directly. It asserts that the ack-challenge and ack are received as well.
async def test_websocket_send_receive_messages(setup_7_nodes, ws_connections):
    # FIXME: for some reason sending NAT-to-NAT does not work in the test setup
    # valid_node_keys = ["1", "2", "3", "4", "5"]
    valid_node_keys = ["1", "2", "3", "4"]
    message_target_count = 50

    for i in range(message_target_count):
        [source_key, target_key] = random.sample(valid_node_keys, 2)

        source_peer = setup_7_nodes[source_key]["peer_id"]
        target_peer = setup_7_nodes[target_key]["peer_id"]

        source_ws = ws_connections[source_key]
        target_ws = ws_connections[target_key]

        body = f"hello msg {i} from peer {source_peer} to peer {target_peer}"

        # we test direct messaging only
        msg = {"cmd": "sendmsg", "args": {"body": body, "recipient": target_peer, "path": []}}

        await source_ws.send(json.dumps(msg))

        try:
            ack_challenge_msg = await asyncio.wait_for(source_ws.recv(), timeout=5)
        except:
            pytest.fail(f"Timeout when receiving ack-challenge of msg {i} from {source_key} to {target_key}")

        assert re.match(r"^ack-challenge:.*$", ack_challenge_msg)

        try:
            inc_msg_rlp = await asyncio.wait_for(target_ws.recv(), timeout=5)
        except:
            pytest.fail(f"Timeout when receiving msg {i} from {source_key} to {target_key}")

        [inc_msg_str, _] = rlp.decode(bytearray([int(c) for c in inc_msg_rlp.split(",")]))
        assert inc_msg_str.decode("unicode_escape") == body

        try:
            ack_msg = await asyncio.wait_for(source_ws.recv(), timeout=5)
        except:
            pytest.fail(f"Timeout when receiving ack of msg {i} from {source_key} to {target_key}")

        assert re.match(r"^ack:.*$", ack_msg)
