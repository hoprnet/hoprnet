import asyncio
import json
import random
import re
import time

import pytest
import websocket
import websockets

from .conftest import API_TOKEN, default_nodes_with_auth, random_distinct_pairs_from
from .node import Node


def url(host, port):
    return f"ws://{host}:{port}/api/v3/messages/websocket"


EXTRA_HEADERS = [("X-Auth-Token", API_TOKEN)]


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_reject_a_connection_without_a_valid_token(peer: str, swarm7: dict[str, Node]):
    ws = websocket.WebSocket()
    try:
        ws.connect(url(swarm7[peer].host_addr, swarm7[peer].api_port))
    except websocket.WebSocketBadStatusException as e:
        assert "401 Unauthorized" in str(e)
    else:
        assert False


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_reject_a_connection_with_an_invalid_token(peer: str, swarm7: dict[str, Node]):
    ws = websocket.WebSocket()
    try:
        ws.connect(
            url(swarm7[peer].host_addr, swarm7[peer].api_port),
            header={"X-Auth-Token": "InvAliD_toKeN"},
        )
    except websocket.WebSocketBadStatusException as e:
        assert "401 Unauthorized" in str(e)
    else:
        assert False, "Failed to raise 401 on invalid token"


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_accept_a_connection_with_an_invalid_token_passed_through_websocket_protocol(
    peer: str, swarm7: dict[str, Node]
):
    ws = websocket.WebSocket()

    try:
        ws.connect(
            url(swarm7[peer].host_addr, swarm7[peer].api_port) + "?apiToken=InvAlidShit",
        )
    except websocket.WebSocketBadStatusException as e:
        assert "401 Unauthorized" in str(e)
    else:
        assert False, "Failed to raise 401 on invalid token"


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_reject_a_connection_with_an_invalid_bearer_token(
    peer: str, swarm7: dict[str, Node]
):
    ws = websocket.WebSocket()
    try:
        ws.connect(
            url(swarm7[peer].host_addr, swarm7[peer].api_port),
            header={"Authorization": "Bearer InvAliD_toKeN"},
        )
    except websocket.WebSocketBadStatusException as e:
        assert "401 Unauthorized" in str(e)
    else:
        assert False, "Failed to raise 401 on invalid token"


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_accept_a_connection_with_a_valid_token(peer: str, swarm7: dict[str, Node]):
    ws = websocket.WebSocket()
    ws.connect(
        url(swarm7[peer].host_addr, swarm7[peer].api_port),
        header={"X-Auth-Token": API_TOKEN},
    )

    time.sleep(0.5)


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_accept_a_connection_with_a_valid_token_passed_through_websocket_protocol(
    peer: str, swarm7: dict[str, Node]
):
    ws = websocket.WebSocket()
    ws.connect(
        url(swarm7[peer].host_addr, swarm7[peer].api_port) + f"?apiToken={API_TOKEN}",
    )

    time.sleep(0.5)


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_accept_a_connection_with_a_valid_bearer_token(peer: str, swarm7: dict[str, Node]):
    ws = websocket.WebSocket()
    ws.connect(
        url(swarm7[peer].host_addr, swarm7[peer].api_port),
        header={"Authorization": "Bearer " + API_TOKEN},
    )

    time.sleep(0.5)


@pytest.mark.parametrize("peer", random.sample(default_nodes_with_auth(), 1))
def test_hoprd_websocket_api_should_reject_connection_on_invalid_path(peer: str, swarm7: dict[str, Node]):
    ws = websocket.WebSocket()
    try:
        ws.connect(
            f"{url(swarm7[peer].host_addr, swarm7[peer].api_port)}/defIniteLY_InVAliD_paTh",
            header={"X-Auth-Token": API_TOKEN},
        )
    except websocket.WebSocketBadStatusException as e:
        assert "404 Not Found" in str(e)
    else:
        assert False, "Failed to raise 404 on invalid path"


@pytest.fixture
async def ws_connections(swarm7: dict[str, Node]):
    async with websockets.connect(
        url(swarm7["1"].host_addr, swarm7["1"].api_port), extra_headers=EXTRA_HEADERS
    ) as ws1, websockets.connect(
        url(swarm7["2"].host_addr, swarm7["2"].api_port), extra_headers=EXTRA_HEADERS
    ) as ws2, websockets.connect(
        url(swarm7["3"].host_addr, swarm7["3"].api_port), extra_headers=EXTRA_HEADERS
    ) as ws3, websockets.connect(
        url(swarm7["4"].host_addr, swarm7["4"].api_port), extra_headers=EXTRA_HEADERS
    ) as ws4:
        yield {"1": ws1, "2": ws2, "3": ws3, "4": ws4}


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(default_nodes_with_auth(), count=1))
async def test_websocket_send_receive_messages(src: str, dest: str, swarm7: dict[str, Node], ws_connections):
    tag = random.randint(30000, 60000)
    message_target_count = 10

    for i in range(message_target_count):
        body = f"hello msg {i} from peer {swarm7[src].peer_id} to peer {swarm7[dest].peer_id}"

        # we test direct messaging only
        msg = {"cmd": "sendmsg", "args": {"body": body, "peerId": swarm7[dest].peer_id, "path": [], "tag": tag}}

        await ws_connections[src].send(json.dumps(msg))

        try:
            ack_challenge_msg = await asyncio.wait_for(ws_connections[src].recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving ack-challenge of msg {i} from {src} to {dest}")
        assert re.match(r"^.*\"message-ack-challenge\".*$", ack_challenge_msg)

        try:
            msg = await asyncio.wait_for(ws_connections[dest].recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving msg {i} from {src} to {dest}")
        assert re.match(f".*{body}.*$", msg)

        try:
            ack_msg = await asyncio.wait_for(ws_connections[src].recv(), timeout=5)
        except Exception:
            pytest.fail(f"Timeout when receiving ack of msg {i} from {src} to {dest}")
        assert re.match(r"^.*\"message-ack\".*$", ack_msg)
