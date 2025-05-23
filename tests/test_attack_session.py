import asyncio
import logging
import os
import random
import string
import pytest
from contextlib import AsyncExitStack, contextmanager

from sdk.python.api.protocol import Protocol
from sdk.python.api.request_objects import SessionCapabilitiesBody
from sdk.python.api.response_objects import Session
from sdk.python.localcluster.constants import TICKET_PRICE_PER_HOP, PORT_BASE
from sdk.python.localcluster.node import Node
from tests.config_nodes import test_session_distruption
from tests.test_session import connect_socket, run_https_server, fetch_data, EchoServer, SocketType

from .conftest import  session_attack_nodes
from .utils import create_channel

HOPR_SESSION_MAX_PAYLOAD_SIZE = 462
STANDARD_MTU_SIZE = 1500

class TestSessionDelayWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [session_attack_nodes()]
    )
    @pytest.mark.parametrize(
        "nodes_config",
        [
            {
                "local1": {},
                "local2": {
                    "HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS": 0,
                    "HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS": 1
                },
                "local3": {} 
            },
            {
                "local1": {},
                "local2": {
                    "HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS": 3000,
                    "HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS": 10
                },
                "local3": {}
            },
            {
                "local1": {},
                "local2": {
                    "HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS": 1000,
                    "HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS": 4000
                },
                "local3": {}
            },
        ],
    )
    async def test_session_http_server_with_random_delays(self, route, swarm3: dict[str, Node], nodes_config: dict[str, dict], config_to_yaml: str):
        """
        Test downloading file from http server over a tcp session while relay node introduces random delay
        """
        file_len = 500

        src_peer = swarm3[route[0]]
        dest_peer = swarm3[route[-1]]
        path = [swarm3[node].peer_id for node in route[1:-1]]

        async with AsyncExitStack() as channels:
            channels_to = [
                channels.enter_async_context(
                    create_channel(
                        swarm3[route[i]], swarm3[route[i + 1]], funding=100 * file_len * TICKET_PRICE_PER_HOP
                    )
                )
                for i in range(len(route) - 1)
            ]
            channels_back = [
                channels.enter_async_context(
                    create_channel(
                        swarm3[route[i]], swarm3[route[i - 1]], funding=100 * file_len * TICKET_PRICE_PER_HOP
                    )
                )
                for i in reversed(range(1, len(route)))
            ]

            await asyncio.gather(*(channels_to + channels_back))
            expected = "".join(random.choices(string.ascii_letters + string.digits, k=file_len))

            with run_https_server(expected) as dst_sock_port:
                session = await src_peer.api.session_client(
                    dest_peer.peer_id,
                    path={"IntermediatePath": path},
                    protocol=Protocol.TCP,
                    target=f"localhost:{dst_sock_port}",
                    capabilities=SessionCapabilitiesBody(retransmission=True, segmentation=True),
                )
                assert session.port is not None, "Failed to open session"
                assert len(await src_peer.api.session_list_clients(Protocol.TCP)) == 1

                response = fetch_data(f"https://localhost:{session.port}/random.txt")

                assert await src_peer.api.session_close_client(session) is True
                assert await src_peer.api.session_list_clients(Protocol.TCP) == []

                assert response is not None
                assert response.text == expected

    @pytest.mark.asyncio
    @pytest.mark.parametrize(
        "route",
        [session_attack_nodes()]
    )
    @pytest.mark.parametrize(
        "nodes_config",
        [
            {
                "local1": {
                    "HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS": 1,
                    "HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS": 1
                },
                "local2": setting,
                "local3": {
                    "HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS": 1,
                    "HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS": 1,
                    "HOPR_DISABLE_ACK_PACKAGES": True
                }
            } for setting in test_session_distruption() # since the setting is too long and diverse, load it from external file
        ]
    )
    async def test_session_communication_with_a_tcp_echo_server_with_random_delay(self, route, swarm3: dict[str, Node], nodes_config: dict[str, dict], config_to_yaml: str):
        """
        Test forming tcp session over the hopr mixnet while relay node introduces random delay
        """
          
        packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
        expected = [f"{i}".ljust(STANDARD_MTU_SIZE) for i in range(packet_count)]

        assert [len(x) for x in expected] == packet_count * [STANDARD_MTU_SIZE]

        src_peer = swarm3[route[0]]
        dest_peer = swarm3[route[-1]]
        path = [swarm3[node].peer_id for node in route[1:-1]]

        async with AsyncExitStack() as channels:
            channels_to = [
                channels.enter_async_context(
                    create_channel(
                        swarm3[route[i]], swarm3[route[i + 1]], funding=20 * packet_count * TICKET_PRICE_PER_HOP
                    )
                )
                for i in range(len(route) - 1)
            ]
            channels_back = [
                channels.enter_async_context(
                    create_channel(
                        swarm3[route[i]], swarm3[route[i - 1]], funding=20 * packet_count * TICKET_PRICE_PER_HOP
                    )
                )
                for i in reversed(range(1, len(route)))
            ]

            await asyncio.gather(*(channels_to + channels_back))

            actual = ""
            with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
                await asyncio.sleep(1.0)

                session = await src_peer.api.session_client(
                    dest_peer.peer_id,
                    path={"IntermediatePath": path},
                    protocol=Protocol.TCP,
                    target=f"localhost:{server.port}",
                    capabilities=SessionCapabilitiesBody(retransmission=True, segmentation=True),
                )

                assert session.port is not None, "Failed to open session"
                assert len(await src_peer.api.session_list_clients(Protocol.TCP)) == 1

                with connect_socket(SocketType.TCP, session.port) as s:
                    s.settimeout(3600)
                    total_sent = 0
                    for message in expected:
                        total_sent = total_sent + s.send(message.encode())

                    while total_sent > 0:
                        chunk = s.recv(min(STANDARD_MTU_SIZE, total_sent))
                        total_sent = total_sent - len(chunk)
                        actual = actual + chunk.decode()

            assert await src_peer.api.session_close_client(session) is True
            assert await src_peer.api.session_list_clients(Protocol.TCP) == []

            local2_conf = nodes_config.get("local2", {})   
            min_delay = local2_conf.get("HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS", "N/A")
        
            metrics = await swarm3[route[1]].api.metrics()
            metrics = "\n".join(line for line in metrics.splitlines() if not line.startswith("#"))
            metrics_dict = dict(line.split(maxsplit=1) for line in metrics.splitlines())
            # logging.info(f"Current node local2 metrics: {metrics}")

            avg_packet_delay = float(metrics_dict.get("hopr_mixer_average_packet_delay", 0))
            packets_relayed = float(metrics_dict.get("hopr_packets_count{type=\"forwarded\"}", 0))
            ack_count_local2 = float(metrics_dict.get("hopr_received_ack_count", 0))
        
            logging.info(f"ack_count_local2: {ack_count_local2}")
            logging.info(f"relay node average packet delay: {avg_packet_delay}");
            logging.info(f"relay node packets relayed: {packets_relayed}");

            # min_delay 10000 is expected to fail
            if min_delay == 10000:
                pytest.xfail("Test is expected to fail due to long min_delay")   

            # assert correct data at the end - long delays can break the tcp sesstion
            # and cause data loss - this is valuable information too
            assert "".join(expected) == actual