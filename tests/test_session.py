import asyncio
import logging
import multiprocessing
import os
import pytest
import socket
import subprocess

from enum import Enum
from contextlib import contextmanager, AsyncExitStack

from .conftest import random_distinct_pairs_from, barebone_nodes, TICKET_PRICE_PER_HOP, fixtures_dir
from .node import Node
from .test_integration import create_channel, shuffled

PARAMETERIZED_SAMPLE_SIZE = 1  # if os.getenv("CI", default="false") == "false" else 3
HOPR_SESSION_MAX_PAYLOAD_SIZE = 462
STANDARD_MTU_SIZE = 1500

# used by nodes to get unique port assignments
PORT_BASE = 19000

class SocketType(Enum):
    TCP = 1
    UDP = 2

class EchoServer:
    def __init__(self, server_type: SocketType, recv_buf_len: int, with_tcpdump: bool = False):
        self.server_type = server_type
        self.port = None
        self.process = None
        self.with_tcpdump = with_tcpdump
        self.tcp_dump_process = None
        self.socket = None
        self.recv_buf_len = recv_buf_len

    def __enter__(self):
        if self.server_type is SocketType.TCP:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        else:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

        self.socket.bind(("127.0.0.1", 0))
        self.port = self.socket.getsockname()[1]

        if self.server_type is SocketType.TCP:
            self.socket.listen()
            self.process = multiprocessing.Process(target=tcp_echo_server_func, args=(self.socket,self.recv_buf_len))
        else:
            self.process = multiprocessing.Process(target=udp_echo_server_func, args=(self.socket,self.recv_buf_len))
        self.process.start()

        if self.with_tcpdump:
            pcap_file = fixtures_dir('test_session').joinpath(f'echo_server_{self.port}.pcap')
            self.tcp_dump_process = subprocess.Popen(['tcpdump', '-i', 'lo0', '-w', f"{pcap_file}.log"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            logging.info(f"running tcpdump, saving to {pcap_file}.log")

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.process.terminate()
        self.socket.close()
        self.socket = None
        self.process = None
        self.port = None

        if self.with_tcpdump:
            self.tcp_dump_process.terminate()
            self.tcp_dump_process = None
            logging.info("terminated tcpdump")
        return True

def tcp_echo_server_func(s,buf_len):
        conn, _addr = s.accept()
        with conn:
            while True:
                data = conn.recv(buf_len)
                conn.sendall(data)
                #logging.info(f"tcp server relayed {len(data)}")


def udp_echo_server_func(s,buf_len):
        while True:
            data, addr = s.recvfrom(buf_len)
            s.sendto(data, addr)
            #logging.info(f"udp server relayed {len(data)}")


@contextmanager
def connect_socket(sock_type: SocketType, port):
    if sock_type is SocketType.TCP:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.connect(("127.0.0.1", port))
    elif sock_type is SocketType.UDP:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    else:
        raise ValueError("Invalid socket type")

    try:
        yield s
    finally:
        s.close()


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_a_tcp_echo_server(
        src: str, dest: str, swarm7: dict[str, Node]
):
    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".ljust(STANDARD_MTU_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [STANDARD_MTU_SIZE]

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    actual = ''
    with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
        # socket.listen does not listen immediately and needs some time to be working
        # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
        await asyncio.sleep(1.0)

        dst_sock_port = server.port
        src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"Hops": 0}, protocol='tcp',
                                                          target=f"localhost:{dst_sock_port}")

        assert src_sock_port is not None, "Failed to open session"
        assert len(await src_peer.api.session_list_clients('tcp')) == 1
        #logging.info(f"session to {dst_sock_port} opened successfully")

        with connect_socket(SocketType.TCP, src_sock_port) as s:
            s.settimeout(20)
            total_sent = 0
            for message in expected:
                total_sent = total_sent + s.send(message.encode())

            while total_sent > 0:
                chunk = s.recv(min(STANDARD_MTU_SIZE, total_sent))
                total_sent = total_sent - len(chunk)
                actual = actual + chunk.decode()

    assert ''.join(expected) == actual

    assert await src_peer.api.session_close_client(protocol='tcp', bound_ip='127.0.0.1', bound_port=src_sock_port) is True
    assert len(await src_peer.api.session_list_clients('tcp')) == 0

@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_communication_over_n_hop_with_a_tcp_echo_server(
        route, swarm7: dict[str, Node]
):
    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".ljust(STANDARD_MTU_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [STANDARD_MTU_SIZE]

    src_peer = swarm7[route[0]]
    dest_peer = swarm7[route[-1]]
    path = [swarm7[node].peer_id for node in route[1:-1]]

    async with AsyncExitStack() as channels:
        channels_to = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=20 * packet_count * TICKET_PRICE_PER_HOP)
            )
            for i in range(len(route) - 1)
        ]
        channels_back = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i - 1]], funding=20 * packet_count * TICKET_PRICE_PER_HOP)
            )
            for i in reversed(range(1, len(route)))
        ]

        await asyncio.gather(*(channels_to + channels_back))

        actual = ''
        with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
            # socket.listen does not listen immediately and needs some time to be working
            # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
            await asyncio.sleep(1.0)

            dst_sock_port = server.port
            src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"IntermediatePath": path}, protocol='tcp',
                                                              target=f"localhost:{dst_sock_port}")

            assert src_sock_port is not None, "Failed to open session"
            assert len(await src_peer.api.session_list_clients('tcp')) == 1
            #logging.info(f"session to {dst_sock_port} opened successfully")

            with connect_socket(SocketType.TCP, src_sock_port) as s:
                s.settimeout(20)
                total_sent = 0
                for message in expected:
                    total_sent = total_sent + s.send(message.encode())

                while total_sent > 0:
                    chunk = s.recv(min(STANDARD_MTU_SIZE, total_sent))
                    total_sent = total_sent - len(chunk)
                    actual = actual + chunk.decode()

        assert ''.join(expected) == actual

        assert await src_peer.api.session_close_client(protocol='tcp', bound_ip='127.0.0.1', bound_port=src_sock_port) is True
        assert len(await src_peer.api.session_list_clients('tcp')) == 0

@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_a_udp_echo_server(
        src: str, dest: str, swarm7: dict[str, Node]
):
    """
    HOPR UDP socket buffers are set to 462 bytes to mimic the underlying MTU of the HOPR protocol.
    """

    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".rjust(HOPR_SESSION_MAX_PAYLOAD_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [HOPR_SESSION_MAX_PAYLOAD_SIZE]

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    actual = []
    with EchoServer(SocketType.UDP, HOPR_SESSION_MAX_PAYLOAD_SIZE) as server:
        await asyncio.sleep(1.0)

        dst_sock_port = server.port
        src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"Hops": 0}, protocol='udp',
                                                          target=f"localhost:{dst_sock_port}")

        assert src_sock_port is not None, "Failed to open session"
        assert len(await src_peer.api.session_list_clients('udp')) == 1
        #logging.info(f"session to {dst_sock_port} opened successfully")

        addr = ('127.0.0.1', src_sock_port)
        with connect_socket(SocketType.UDP, None) as s:
            s.settimeout(20)
            total_sent = 0
            for message in expected:
                total_sent = total_sent + s.sendto(message.encode(), addr)
                await asyncio.sleep(0.01) # UDP has no flow-control, so we must insert an artificial gap

            while total_sent > 0:
                chunk, _ = s.recvfrom(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
                total_sent = total_sent - len(chunk)
                actual.append(chunk.decode())

    actual.sort()
    expected.sort()
    assert actual == expected

    assert await src_peer.api.session_close_client(protocol='udp', bound_ip='127.0.0.1', bound_port=src_sock_port) is True
    assert len(await src_peer.api.session_list_clients('udp')) == 0

@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_communication_over_n_hop_with_a_udp_echo_server(
        route, swarm7: dict[str, Node]
):
    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".ljust(HOPR_SESSION_MAX_PAYLOAD_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [HOPR_SESSION_MAX_PAYLOAD_SIZE]

    src_peer = swarm7[route[0]]
    dest_peer = swarm7[route[-1]]
    path = [swarm7[node].peer_id for node in route[1:-1]]

    async with AsyncExitStack() as channels:
        channels_to = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=packet_count * TICKET_PRICE_PER_HOP)
            )
            for i in range(len(route) - 1)
        ]
        channels_back = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i - 1]], funding=packet_count * TICKET_PRICE_PER_HOP)
            )
            for i in reversed(range(1, len(route)))
        ]

        await asyncio.gather(*(channels_to + channels_back))

        actual = []
        with EchoServer(SocketType.UDP, HOPR_SESSION_MAX_PAYLOAD_SIZE, True) as server:
            await asyncio.sleep(1.0)

            dst_sock_port = server.port
            src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"IntermediatePath": path}, protocol='udp',
                                                              target=f"localhost:{dst_sock_port}")

            assert src_sock_port is not None, "Failed to open session"
            assert len(await src_peer.api.session_list_clients('udp')) == 1
            #logging.info(f"session to {dst_sock_port} opened successfully")

            addr = ('127.0.0.1', src_sock_port)
            with connect_socket(SocketType.UDP, None) as s:
                s.settimeout(20)
                total_sent = 0
                for message in expected:
                    total_sent = total_sent + s.sendto(message.encode(), addr)
                    await asyncio.sleep(0.01) # UDP has no flow-control, so we must insert an artificial gap

                while total_sent > 0:
                    chunk, _ = s.recvfrom(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
                    total_sent = total_sent - len(chunk)
                    actual.append(chunk.decode())

        actual.sort()
        expected.sort()
        assert actual == expected

        assert await src_peer.api.session_close_client(protocol='udp', bound_ip='127.0.0.1', bound_port=src_sock_port) is True
        assert len(await src_peer.api.session_list_clients('udp')) == 0

