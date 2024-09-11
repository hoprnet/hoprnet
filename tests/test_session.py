import asyncio
import multiprocessing
import os
import pytest
import socket

from enum import Enum
from contextlib import contextmanager
from typing import Callable

from .conftest import random_distinct_pairs_from, barebone_nodes
from .node import Node

PARAMETERIZED_SAMPLE_SIZE = 1  # if os.getenv("CI", default="false") == "false" else 3
ECHO_SERVER_PORT = 10101
HOPR_SESSION_MAX_PAYLOAD_SIZE = 462

# used by nodes to get unique port assignments
PORT_BASE = 19000

def tcp_echo_server_func(port: int):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", port))
        s.listen()
        conn, _addr = s.accept()
        with conn:
            while True:
                data = conn.recv(HOPR_SESSION_MAX_PAYLOAD_SIZE)
                conn.sendall(data)


def udp_echo_server_func(port: int):
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
        s.bind(("127.0.0.1", port))

        while True:
            data, addr = s.recvfrom(HOPR_SESSION_MAX_PAYLOAD_SIZE)
            s.sendto(data, addr)


@contextmanager
def run_server(server_func: Callable[..., object], port: int):
    process = multiprocessing.Process(target=server_func, args=(port,))
    process.start()
    try:
        yield port
    finally:
        process.terminate()

class SocketType(Enum):
    TCP = 1
    UDP = 2

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
    """
    HOPR TCP socket buffers are set to 462 bytes to mimic the underlying MTU of the HOPR protocol.
    """

    packet_count = 1000 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".ljust(HOPR_SESSION_MAX_PAYLOAD_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [HOPR_SESSION_MAX_PAYLOAD_SIZE]

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"Hops": 0}, protocol='tcp',
                                                      target=f"localhost:{ECHO_SERVER_PORT}")

    assert len(await src_peer.api.session_list_clients('tcp')) == 1

    actual = ''

    with run_server(tcp_echo_server_func, ECHO_SERVER_PORT):
        # socket.listen does not listen immediately and needs some time to be working
        # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
        await asyncio.sleep(1.0)

        with connect_socket(SocketType.TCP, src_sock_port) as s:
            s.settimeout(20)
            total_sent = 0
            for message in expected:
                total_sent = total_sent + s.send(message.encode())

            while total_sent > 0:
                chunk = s.recv(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
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

    src_sock_port = await src_peer.api.session_client(dest_peer.peer_id, path={"Hops": 0}, protocol='udp',
                                                      target=f"localhost:{ECHO_SERVER_PORT}")

    assert len(await src_peer.api.session_list_clients('udp')) == 1

    actual = []

    with run_server(udp_echo_server_func, ECHO_SERVER_PORT):
        await asyncio.sleep(1.0)

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

