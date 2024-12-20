import asyncio
import http.server
import logging
import multiprocessing
import os
import random
import re
import socket
import ssl
import string
import subprocess
import threading
import time
from contextlib import AsyncExitStack, contextmanager
from datetime import datetime, timedelta
from enum import Enum
from functools import partial

import pytest
import requests
import urllib3
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509.oid import NameOID

from sdk.python.api.protocol import Protocol
from sdk.python.api.request_objects import SessionCapabilitiesBody
from sdk.python.api.response_objects import Session
from sdk.python.localcluster.constants import MAIN_DIR, TICKET_PRICE_PER_HOP
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes, random_distinct_pairs_from
from .utils import PARAMETERIZED_SAMPLE_SIZE, create_channel, shuffled

HOPR_SESSION_MAX_PAYLOAD_SIZE = 462
STANDARD_MTU_SIZE = 1500


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
            self.process = multiprocessing.Process(target=tcp_echo_server_func, args=(self.socket, self.recv_buf_len))
        else:
            self.process = multiprocessing.Process(target=udp_echo_server_func, args=(self.socket, self.recv_buf_len))
        self.process.start()

        # If needed, tcp dump can be started to catch traffic on the local interface
        if self.with_tcpdump:
            pcap_file = MAIN_DIR.joinpath("test_session", f"echo_server_{self.port}.pcap")
            self.tcp_dump_process = subprocess.Popen(
                ["sudo", "tcpdump", "-i", "lo", "-w", f"{pcap_file}.log"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            logging.info(f"running tcpdump, saving to {pcap_file}.log")

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.process.terminate()
        self.socket.close()
        self.socket = None
        self.process = None
        self.port = None

        if self.with_tcpdump:
            logging.info("killing tcp dump")
            stdout, stderr = self.tcp_dump_process.communicate()
            self.tcp_dump_process.kill()
            self.tcp_dump_process = None
            logging.info(f"terminated tcpdump: {stdout}, {stderr}")
        return True


def tcp_echo_server_func(s, buf_len):
    conn, _addr = s.accept()
    with conn:
        while True:
            data = conn.recv(buf_len)
            conn.sendall(data)


def udp_echo_server_func(s, buf_len):
    while True:
        data, addr = s.recvfrom(buf_len)
        s.sendto(data, addr)


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


def fetch_data(url: str):
    # Suppress only the single InsecureRequestWarning from urllib3 needed for self-signed certs
    urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

    try:
        # set verify=False for self-signed certs
        response = requests.get(url, verify=False)
        response.raise_for_status()
        return response
    except requests.exceptions.RequestException as e:
        logging.error(f"HTTP request failed: {e}")
        return None


def generate_self_signed_cert(cert_file_with_key):
    key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    subject = issuer = x509.Name(
        [
            x509.NameAttribute(NameOID.COUNTRY_NAME, "CH"),
            x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, "Switzerland"),
            x509.NameAttribute(NameOID.LOCALITY_NAME, "Zurich"),
            x509.NameAttribute(NameOID.ORGANIZATION_NAME, "HOPR"),
            x509.NameAttribute(NameOID.COMMON_NAME, "localhost"),
        ]
    )

    cert = (
        x509.CertificateBuilder()
        .subject_name(subject)
        .issuer_name(issuer)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(datetime.utcnow())
        .not_valid_after(datetime.utcnow() + timedelta(days=365))
        .add_extension(
            x509.SubjectAlternativeName([x509.DNSName("localhost")]),
            critical=False,
        )
        .sign(key, hashes.SHA256())
    )

    # Combine the private key and certificate into a single PEM file
    with open(cert_file_with_key, "wb") as f:
        f.write(
            key.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.TraditionalOpenSSL,
                encryption_algorithm=serialization.NoEncryption(),
            )
        )
        f.write(cert.public_bytes(serialization.Encoding.PEM))


class CustomHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, content=None, *args, **kwargs):
        self.content = content
        super().__init__(*args, **kwargs)

    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write(self.content.encode("utf-8"))
        time.sleep(1)  # add an artificial delay


@contextmanager
def run_https_server(served_text_content):
    cert_file = "cert.pem"

    # Generate the certificate and key if they don't exist
    if not os.path.exists(cert_file):
        logging.debug("generating self-signed certificate and key...")
        generate_self_signed_cert(cert_file)

    # Create a handler class with the content passed
    handler_class = partial(CustomHTTPRequestHandler, served_text_content)

    # Set up the HTTP server with a random port and SSL context
    httpd = http.server.HTTPServer(("localhost", 0), handler_class)
    ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ssl_context.load_cert_chain(certfile=cert_file, keyfile=cert_file)
    httpd.socket = ssl_context.wrap_socket(httpd.socket, server_side=True)

    # Get the random port assigned to the server
    port = httpd.server_address[1]
    server_thread = threading.Thread(target=httpd.serve_forever)
    try:
        server_thread.start()
        logging.debug(f"serving on https://localhost:{port}")
        yield port
    finally:
        logging.debug("shutting down the HTTP server...")
        httpd.shutdown()
        httpd.server_close()
        server_thread.join()
        if os.path.exists(cert_file):
            os.remove(cert_file)


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_a_tcp_echo_server(src: str, dest: str, swarm7: dict[str, Node]):
    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".ljust(STANDARD_MTU_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [STANDARD_MTU_SIZE]

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    actual = ""
    with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
        # socket.listen does not listen immediately and needs some time to be working
        # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
        await asyncio.sleep(1.0)

        dst_sock_port = server.port
        session = await src_peer.api.session_client(
            dest_peer.peer_id, path={"Hops": 0}, protocol=Protocol.TCP, target=f"localhost:{dst_sock_port}"
        )

        assert session.port is not None, "Failed to open session"
        assert len(await src_peer.api.session_list_clients(Protocol.TCP)) == 1

        with connect_socket(SocketType.TCP, session.port) as s:
            s.settimeout(20)
            total_sent = 0
            for message in expected:
                total_sent = total_sent + s.send(message.encode())

            while total_sent > 0:
                chunk = s.recv(min(STANDARD_MTU_SIZE, total_sent))
                total_sent = total_sent - len(chunk)
                actual = actual + chunk.decode()

    assert await src_peer.api.session_close_client(session) is True
    assert await src_peer.api.session_list_clients(Protocol.TCP) == []

    assert "".join(expected) == actual


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_communication_over_n_hop_with_a_tcp_echo_server(route, swarm7: dict[str, Node]):
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

        actual = ""
        with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
            # socket.listen does not listen immediately and needs some time to be working
            # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
            await asyncio.sleep(1.0)

            dst_sock_port = server.port
            session = await src_peer.api.session_client(
                dest_peer.peer_id,
                path={"IntermediatePath": path},
                protocol=Protocol.TCP,
                target=f"localhost:{dst_sock_port}",
            )

            assert session.port is not None, "Failed to open session"
            assert len(await src_peer.api.session_list_clients(Protocol.TCP)) == 1

            with connect_socket(SocketType.TCP, session.port) as s:
                s.settimeout(20)
                total_sent = 0
                for message in expected:
                    total_sent = total_sent + s.send(message.encode())

                while total_sent > 0:
                    chunk = s.recv(min(STANDARD_MTU_SIZE, total_sent))
                    total_sent = total_sent - len(chunk)
                    actual = actual + chunk.decode()

        assert await src_peer.api.session_close_client(session) is True
        assert await src_peer.api.session_list_clients(Protocol.TCP) == []

        assert "".join(expected) == actual


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_a_udp_echo_server(src: str, dest: str, swarm7: dict[str, Node]):
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
        session = await src_peer.api.session_client(
            dest_peer.peer_id, path={"Hops": 0}, protocol=Protocol.UDP, target=f"localhost:{dst_sock_port}"
        )

        assert session.port is not None, "Failed to open session"
        assert len(await src_peer.api.session_list_clients(Protocol.UDP)) == 1
        # logging.info(f"session to {dst_sock_port} opened successfully")

        addr = ("127.0.0.1", session.port)
        with connect_socket(SocketType.UDP, None) as s:
            s.settimeout(20)
            total_sent = 0
            for message in expected:
                total_sent = total_sent + s.sendto(message.encode(), addr)
                # UDP has no flow-control, so we must insert an artificial gap
                await asyncio.sleep(0.01)

            while total_sent > 0:
                chunk, _ = s.recvfrom(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
                total_sent = total_sent - len(chunk)
                # Adapt for situations when data arrive completely unordered (also within the buffer)
                actual.extend([m for m in re.split(r"\s+", chunk.decode().strip()) if len(m) > 0])

    expected = [msg.strip() for msg in expected]

    actual.sort()
    expected.sort()

    assert await src_peer.api.session_close_client(session) is True
    assert await src_peer.api.session_list_clients(Protocol.UDP) == []

    assert actual == expected


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_udp_loopback_service(src: str, dest: str, swarm7: dict[str, Node]):
    """
    HOPR UDP socket buffers are set to 462 bytes to mimic the underlying MTU of the HOPR protocol.
    """

    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".rjust(HOPR_SESSION_MAX_PAYLOAD_SIZE) for i in range(packet_count)]

    assert [len(x) for x in expected] == packet_count * [HOPR_SESSION_MAX_PAYLOAD_SIZE]

    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    actual = []

    # Service 0 session will loop back all the data at Exit back to the Entry
    # Therefore, we do not need the Echo service here
    session: Session | None = await src_peer.api.session_client(
        dest_peer.peer_id, path={"Hops": 0}, protocol=Protocol.UDP, target="0", service=True
    )

    assert session.port is not None, "Failed to open session"
    assert len(await src_peer.api.session_list_clients(Protocol.UDP)) == 1
    # logging.info(f"session to {dst_sock_port} opened successfully")

    addr = ("127.0.0.1", session.port)
    with connect_socket(SocketType.UDP, None) as s:
        s.settimeout(20)
        total_sent = 0
        for message in expected:
            total_sent = total_sent + s.sendto(message.encode(), addr)
            # UDP has no flow-control, so we must insert an artificial gap
            await asyncio.sleep(0.01)

        while total_sent > 0:
            chunk, _ = s.recvfrom(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
            total_sent = total_sent - len(chunk)
            # Adapt for situations when data arrive completely unordered (also within the buffer)
            actual.extend([m for m in re.split(r"\s+", chunk.decode().strip()) if len(m) > 0])

    expected = [msg.strip() for msg in expected]

    actual.sort()
    expected.sort()

    assert await src_peer.api.session_close_client(session)
    assert await src_peer.api.session_list_clients(Protocol.UDP) == []

    assert actual == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_communication_over_n_hop_with_a_udp_echo_server(route, swarm7: dict[str, Node]):
    packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
    expected = [f"{i}".rjust(HOPR_SESSION_MAX_PAYLOAD_SIZE) for i in range(packet_count)]

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
        with EchoServer(SocketType.UDP, HOPR_SESSION_MAX_PAYLOAD_SIZE) as server:
            await asyncio.sleep(1.0)

            dst_sock_port = server.port
            session = await src_peer.api.session_client(
                dest_peer.peer_id,
                path={"IntermediatePath": path},
                protocol=Protocol.UDP,
                target=f"localhost:{dst_sock_port}",
            )

            assert session.port is not None, "Failed to open session"
            assert len(await src_peer.api.session_list_clients(Protocol.UDP)) == 1
            # logging.info(f"session to {dst_sock_port} opened successfully")

            addr = ("127.0.0.1", session.port)
            with connect_socket(SocketType.UDP, None) as s:
                s.settimeout(20)
                total_sent = 0
                for message in expected:
                    total_sent = total_sent + s.sendto(message.encode(), addr)
                    # UDP has no flow-control, so we must insert an artificial gap
                    await asyncio.sleep(0.01)

                while total_sent > 0:
                    chunk, _ = s.recvfrom(min(HOPR_SESSION_MAX_PAYLOAD_SIZE, total_sent))
                    total_sent = total_sent - len(chunk)
                    # Adapt for situations when data arrive completely unordered (also within the buffer)
                    actual.extend([m for m in re.split(r"\s+", chunk.decode().strip()) if len(m) > 0])

        expected = [msg.strip() for msg in expected]

        actual.sort()
        expected.sort()

        assert await src_peer.api.session_close_client(session) is True
        assert await src_peer.api.session_list_clients(Protocol.UDP) == []

        assert actual == expected


@pytest.mark.asyncio
@pytest.mark.parametrize("src,dest", random_distinct_pairs_from(barebone_nodes(), count=PARAMETERIZED_SAMPLE_SIZE))
async def test_session_communication_with_an_https_server(src: str, dest: str, swarm7: dict[str, Node]):
    file_len = 500
    src_peer = swarm7[src]
    dest_peer = swarm7[dest]

    # Generate random text content to be served
    expected = "".join(random.choices(string.ascii_letters + string.digits, k=file_len))

    with run_https_server(expected) as dst_sock_port:
        session = await src_peer.api.session_client(
            dest_peer.peer_id,
            path={"Hops": 0},
            protocol=Protocol.TCP,
            target=f"localhost:{dst_sock_port}",
            sealed_target=True,
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
    [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_communication_over_n_hop_with_an_https_server(route, swarm7: dict[str, Node]):
    file_len = 500

    src_peer = swarm7[route[0]]
    dest_peer = swarm7[route[-1]]
    path = [swarm7[node].peer_id for node in route[1:-1]]

    async with AsyncExitStack() as channels:
        channels_to = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i + 1]], funding=100 * file_len * TICKET_PRICE_PER_HOP)
            )
            for i in range(len(route) - 1)
        ]
        channels_back = [
            channels.enter_async_context(
                create_channel(swarm7[route[i]], swarm7[route[i - 1]], funding=100 * file_len * TICKET_PRICE_PER_HOP)
            )
            for i in reversed(range(1, len(route)))
        ]

        await asyncio.gather(*(channels_to + channels_back))

        # Generate random text content to be served
        expected = "".join(random.choices(string.ascii_letters + string.digits, k=file_len))

        with run_https_server(expected) as dst_sock_port:
            session = await src_peer.api.session_client(
                dest_peer.peer_id,
                path={"IntermediatePath": path},
                protocol=Protocol.TCP,
                target=f"localhost:{dst_sock_port}",
            )
            assert session.port is not None, "Failed to open session"
            assert len(await src_peer.api.session_list_clients(Protocol.TCP)) == 1

            response = fetch_data(f"https://localhost:{session.port}/random.txt")

            assert await src_peer.api.session_close_client(session) is True
            assert await src_peer.api.session_list_clients(Protocol.TCP) == []

            assert response is not None
            assert response.text == expected


@pytest.mark.skipif(
    os.environ.get("HOPR_TEST_RUNNING_WIREGUARD_TUNNEL") is None, reason="Wireguard tunnel with for hoprnet running"
)
@pytest.mark.asyncio
@pytest.mark.parametrize(
    "route",
    [barebone_nodes()[:3]],
    # [shuffled(barebone_nodes())[:3] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
    # + [shuffled(nodes())[:5] for _ in range(PARAMETERIZED_SAMPLE_SIZE)],
)
async def test_session_with_wireguard_tunnel(route, swarm7: dict[str, Node]):
    packet_count = 10_000_000
    wireguard_tunnel = os.environ.get("HOPR_TEST_RUNNING_WIREGUARD_TUNNEL")

    src_peer = swarm7[route[0]]
    dest_peer = swarm7[route[-1]]
    path = [swarm7[node].peer_id for node in route[1:-1]]

    logging.info(f"Opening channels for route '{route}'")

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

        # sleep to wait for the socket to be active
        await asyncio.sleep(1.0)

        logging.info(f"Opening session for route '{route}'")

        session = await src_peer.api.session_client(
            dest_peer.peer_id,
            path={"IntermediatePath": path},
            protocol=Protocol.UDP,
            target=wireguard_tunnel,
            listen_on="127.0.0.1:60006",
            capabilities=SessionCapabilitiesBody(segment=True),
        )

        assert session.port is not None, "Failed to open session"
        assert len(await src_peer.api.session_list_clients(Protocol.UDP)) == 1

        logging.info("Test ready for execution")

        # TODO: Placeholder for actual test
        await asyncio.sleep(3600)
