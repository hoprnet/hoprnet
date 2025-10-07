import asyncio
import time
from contextlib import closing

import pytest
import websocket
from websockets.asyncio.client import connect

from sdk.python.localcluster.node import Node

from .conftest import nodes_with_auth, to_ws_url
from .test_session import STANDARD_MTU_SIZE, EchoServer, SocketType
from .utils import random_distinct_pairs_from

DEFAULT_ARGS = [
    ("hops", 0),
    ("capabilities", "Segmentation"),
    ("capabilities", "Retransmission"),
    ("target", "127.0.0.1:4677"),
]


@pytest.mark.usefixtures("swarm7_reset")
class TestWebsocketWithSwarm:
    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_reject_a_connection_without_a_valid_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            try:
                url = to_ws_url(
                    swarm7[src].host_addr,
                    swarm7[src].api_port,
                    args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
                )
                ws.connect(url)
            except websocket.WebSocketBadStatusException as e:
                assert "401 Unauthorized" in str(e)
            else:
                pytest.fail("Should fail with 401 Unauthorized")  # ty: ignore[call-non-callable]

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_reject_a_connection_with_an_invalid_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            try:
                url = to_ws_url(
                    swarm7[src].host_addr,
                    swarm7[src].api_port,
                    args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
                )
                ws.connect(url, header={"X-Auth-Token": "InvAliD_toKeN"})
            except websocket.WebSocketBadStatusException as e:
                assert "401 Unauthorized" in str(e)
            else:
                pytest.fail("Should fail with 401 Unauthorized")  # ty: ignore[call-non-callable]

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_not_accept_a_connection_with_an_invalid_token_passed_as_query_param(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            try:
                url = to_ws_url(
                    swarm7[src].host_addr,
                    swarm7[src].api_port,
                    args=[("apiToken", "InvAlid_Token")] + DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
                )
                ws.connect(url)
            except websocket.WebSocketBadStatusException as e:
                assert "401 Unauthorized" in str(e)
            else:
                pytest.fail("Should fail with 401 Unauthorized")  # ty: ignore[call-non-callable]

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_reject_a_connection_with_an_invalid_bearer_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            try:
                url = to_ws_url(
                    swarm7[src].host_addr,
                    swarm7[src].api_port,
                    args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
                )
                ws.connect(url, header={"Authorization": "Bearer InvAliD_toKeN"})
            except websocket.WebSocketBadStatusException as e:
                assert "401 Unauthorized" in str(e)
            else:
                pytest.fail("Should fail with 401 Unauthorized")  # ty: ignore[call-non-callable]

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_accept_a_connection_with_a_valid_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            url = to_ws_url(
                swarm7[src].host_addr,
                swarm7[src].api_port,
                args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
            )
            ws.connect(url, header={"X-Auth-Token": swarm7[src].api_token})

        time.sleep(0.5)

    @pytest.mark.xfail(
        reason="This test is expected to fail due to a bug in the axum code,"
        + "where the query is not parsed for the token"
    )
    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_accept_a_connection_with_a_query_param_passed_valid_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            url = to_ws_url(
                swarm7[src].host_addr,
                swarm7[src].api_port,
                args=[("apiToken", swarm7[src].api_token)]
                + DEFAULT_ARGS
                + [("destination", f"{swarm7[dest].address}")],
            )
            ws.connect(url)

        time.sleep(0.5)

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_accept_a_connection_with_a_valid_bearer_token(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        with closing(websocket.WebSocket()) as ws:
            url = to_ws_url(
                swarm7[src].host_addr,
                swarm7[src].api_port,
                args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
            )
            ws.connect(url, header={"Authorization": "Bearer " + swarm7[src].api_token})

        time.sleep(0.5)

    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    def test_hoprd_websocket_api_should_reject_connection_on_invalid_path(
        self, src: str, dest: str, swarm7: dict[str, Node]
    ):
        ws = websocket.WebSocket()
        try:
            url = to_ws_url(
                swarm7[src].host_addr,
                f"{swarm7[src].api_port}/defIniteLY_InVAliD_paTh",
                args=DEFAULT_ARGS + [("destination", f"{swarm7[dest].address}")],
            )
            ws.connect(url, header={"X-Auth-Token": swarm7[src].api_token})
        except websocket.WebSocketBadStatusException as e:
            assert "404 Not Found" in str(e)
        else:
            pytest.fail("Should fail with 404 Not Found")  # ty: ignore[call-non-callable]

    @pytest.mark.asyncio
    @pytest.mark.parametrize("src,dest", random_distinct_pairs_from(nodes_with_auth(), count=1))
    async def test_websocket_send_receive_messages(self, src: str, dest: str, swarm7: dict[str, Node]):
        message_target_count = 10

        with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
            async with connect(
                to_ws_url(
                    swarm7[src].host_addr,
                    swarm7[src].api_port,
                    args=[("target", f"127.0.0.1:{server.port}")]
                    + DEFAULT_ARGS
                    + [("destination", f"{swarm7[dest].address}")],
                ),
                additional_headers=[("X-Auth-Token", swarm7[src].api_token)],
            ) as ws:
                for i in range(message_target_count):
                    body = f"hello msg #{i} from peer {swarm7[src].address} to peer {swarm7[dest].address}"

                    await ws.send(body.encode())

                    try:
                        msg = await asyncio.wait_for(ws.recv(), timeout=5)
                    except Exception:
                        pytest.fail(
                            f"Timeout when receiving msg {i} from {src} to {dest}"
                        )  # ty: ignore[call-non-callable]
                    assert body == msg.decode(), "sent data content should be identical"
