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
from contextlib import contextmanager
from datetime import datetime, timedelta, timezone
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
from sdk.python.localcluster.constants import MAIN_DIR
from sdk.python.localcluster.node import Node

from .conftest import barebone_nodes
from .utils import (
    HoprSession,
    basic_send_and_receive_packets_over_single_route,
    create_bidirectional_channels_for_route,
    get_ticket_price,
    make_routes,
)

STANDARD_MTU_SIZE = 1500
DOWNLOAD_FILE_SIZE = 800


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
    except requests.RequestException as e:
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
        .not_valid_before(datetime.now(timezone.utc))
        .not_valid_after(datetime.now(timezone.utc) + timedelta(days=365))
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


@pytest.mark.usefixtures("swarm7_reset")
class TestSessionWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([0, 1], barebone_nodes()))
    async def test_session_communication_over_n_hop_with_a_tcp_echo_server(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
        surb_pre_buffer = 6000

        expected = [f"{i}".ljust(STANDARD_MTU_SIZE) for i in range(packet_count)]
        actual = ""

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route],
            20 * (surb_pre_buffer + packet_count) * ticket_price,
            20 * packet_count * ticket_price,
        ):
            with EchoServer(SocketType.TCP, STANDARD_MTU_SIZE) as server:
                # socket.listen does not listen immediately and needs some time to be working
                # otherwise a `ConnectionRefusedError: [Errno 61] Connection refused` will be encountered
                await asyncio.sleep(1.0)

                # Session uses Response buffer and Exit egress rate control
                async with HoprSession(
                    Protocol.TCP,
                    src=swarm7[route[0]],
                    dest=swarm7[route[-1]],
                    fwd_path={"IntermediatePath": [swarm7[hop].address for hop in route[1:-1]]},
                    return_path={"IntermediatePath": [swarm7[hop].address for hop in route[-2:0:-1]]},
                    capabilities=SessionCapabilitiesBody(retransmission=True, segmentation=True),
                    target_port=server.port,
                ) as session:
                    assert len(await swarm7[route[0]].api.session_list_clients(Protocol.TCP)) == 1

                    logging.debug(f"Session opened - sending {len(expected)} packets")
                    with session.client_socket() as sock:
                        sock.settimeout(20)
                        total_sent = 0
                        for message in expected:
                            total_sent = total_sent + sock.send(message.encode())

                        while total_sent > 0:
                            chunk = sock.recv(min(STANDARD_MTU_SIZE, total_sent))
                            total_sent = total_sent - len(chunk)
                            actual = actual + chunk.decode()
                        logging.debug(f"received: {len(actual)}")

                assert await swarm7[route[0]].api.session_list_clients(Protocol.TCP) == []
                assert "".join(expected) == actual

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([0, 1], barebone_nodes()))
    async def test_session_communication_over_n_hop_with_a_udp_echo_server(self, route, swarm7: dict[str, Node]):
        packet_count = 100 if os.getenv("CI", default="false") == "false" else 50
        ticket_price = await get_ticket_price(swarm7[route[0]])

        assert await swarm7[route[0]].api.session_list_clients(Protocol.UDP) == []

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route],
            (packet_count + 2) * ticket_price,
            (packet_count + 2) * ticket_price,
        ):
            await basic_send_and_receive_packets_over_single_route(
                packet_count,
                [swarm7[hop] for hop in route],
            )

        assert await swarm7[route[0]].api.session_list_clients(Protocol.UDP) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([0], barebone_nodes()))
    async def test_session_parameter_reconfiguration(self, route, swarm7: dict[str, Node]):
        async with HoprSession(
            Protocol.UDP,
            src=swarm7[route[0]],
            dest=swarm7[route[-1]],
            fwd_path={"IntermediatePath": [swarm7[hop].address for hop in route[1:-1]]},
            return_path={"IntermediatePath": [swarm7[hop].address for hop in route[-2:0:-1]]},
            use_response_buffer="1002 kB",  # currently set to be the exact multiple of the MTU
            capabilities=SessionCapabilitiesBody(retransmission=False, segmentation=False),
        ) as session:
            assert len(session.active_clients) == 1
            session_id = session.active_clients[0]

            entry = await swarm7[route[0]].api.session_list_clients(Protocol.UDP)
            assert len(entry) == 1
            assert len(entry[0].active_clients) == 1
            assert entry[0].active_clients[0] == session_id

            cfg = await swarm7[route[0]].api.session_get_config(session_id)
            assert cfg is not None
            assert cfg.response_buffer == "978.5 KiB"  # correction from kB to KiB

            cfg.response_buffer = "2004 kB"
            await swarm7[route[0]].api.session_set_config(session_id, cfg)

            cfg = await swarm7[route[0]].api.session_get_config(session_id)
            assert cfg is not None
            assert cfg.response_buffer == "1.9 MiB"  # correction from kB to MiB

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([0], barebone_nodes()))
    async def test_session_communication_with_udp_loopback_service(self, route, swarm7: dict[str, Node]):
        packet_count = 100 if os.getenv("CI", default="false") == "false" else 50

        # Session uses NO Response buffer and NO Exit egress rate control
        async with HoprSession(
            Protocol.UDP,
            src=swarm7[route[0]],
            dest=swarm7[route[-1]],
            fwd_path={"IntermediatePath": [swarm7[hop].address for hop in route[1:-1]]},
            return_path={"IntermediatePath": [swarm7[hop].address for hop in route[-2:0:-1]]},
            loopback=True,
            use_response_buffer=None,
            capabilities=SessionCapabilitiesBody(retransmission=False, segmentation=False, no_rate_control=True),
        ) as session:
            assert len(await swarm7[route[0]].api.session_list_clients(Protocol.UDP)) == 1

            # Leave some space for SURBs in the packet, because no response buffer is used
            packet_len = int(session.hopr_mtu / 2)
            expected = [f"{i}".ljust(packet_len) for i in range(packet_count)]
            actual = []
            with session.client_socket() as sock:
                sock.settimeout(20)
                total_sent = 0
                for message in expected:
                    total_sent = total_sent + sock.sendto(message.encode(), ("127.0.0.1", session.listen_port))
                    # UDP has no flow-control, so we must insert an artificial gap
                    await asyncio.sleep(0.01)

                logging.debug(f"total sent: {total_sent}")

                # Receiving data on the same socket, because it is a loopback
                while total_sent > 0:
                    chunk, _ = sock.recvfrom(min(packet_len, total_sent))
                    total_sent = total_sent - len(chunk)
                    logging.debug(f"received: {len(chunk)}, remaining: {total_sent}")

                    # Adapt for situations when data arrive completely unordered (also within the buffer)
                    actual.extend([m for m in re.split(r"\s+", chunk.decode().strip()) if len(m) > 0])

                expected = [msg.strip() for msg in expected]

            actual.sort()
            expected.sort()
            assert actual == expected

        assert await swarm7[route[0]].api.session_list_clients(Protocol.UDP) == []

    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([0, 1], barebone_nodes()))
    async def test_session_communication_over_n_hop_with_an_https_server(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        surb_pre_buffer = 6000

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route],
            100 * (surb_pre_buffer + DOWNLOAD_FILE_SIZE) * ticket_price,
            100 * DOWNLOAD_FILE_SIZE * ticket_price,
        ):
            # Generate random text content to be served
            expected = "".join(random.choices(string.ascii_letters + string.digits, k=DOWNLOAD_FILE_SIZE))

            # Session uses Response buffer and Exit egress rate control
            with run_https_server(expected) as dst_sock_port:
                async with HoprSession(
                    Protocol.TCP,
                    src=swarm7[route[0]],
                    dest=swarm7[route[-1]],
                    fwd_path={"IntermediatePath": [swarm7[hop].address for hop in route[1:-1]]},
                    return_path={"IntermediatePath": [swarm7[hop].address for hop in route[-2:0:-1]]},
                    capabilities=SessionCapabilitiesBody(retransmission=True, segmentation=True),
                    target_port=dst_sock_port,
                ) as session:
                    assert len(await swarm7[route[0]].api.session_list_clients(Protocol.TCP)) == 1
                    response = fetch_data(f"https://localhost:{session.listen_port}/random.txt")
                    assert response is not None
                    assert response.text == expected

            assert await swarm7[route[0]].api.session_list_clients(Protocol.TCP) == []

    @pytest.mark.skipif(
        os.environ.get("HOPR_TEST_RUNNING_WIREGUARD_TUNNEL_SERVER_PORT") is None,
        reason="Wireguard tunnel with for hoprnet running",
    )
    @pytest.mark.asyncio
    @pytest.mark.parametrize("route", make_routes([1], barebone_nodes()))
    async def test_session_with_wireguard_tunnel(self, route, swarm7: dict[str, Node]):
        ticket_price = await get_ticket_price(swarm7[route[0]])
        packet_count = 10_000_000
        wireguard_tunnel_port = os.environ.get("HOPR_TEST_RUNNING_WIREGUARD_TUNNEL_SERVER_PORT")

        logging.info(f"Opening channels for route '{route}'")

        async with create_bidirectional_channels_for_route(
            [swarm7[hop] for hop in route], packet_count * ticket_price, packet_count * ticket_price
        ):
            # Session uses Response buffer and Exit egress rate control
            logging.info(f"Opening session for route '{route}'")
            async with HoprSession(
                Protocol.UDP,
                src=swarm7[route[0]],
                dest=swarm7[route[-1]],
                fwd_path={"IntermediatePath": [swarm7[hop].address for hop in route[1:-1]]},
                return_path={"IntermediatePath": [swarm7[hop].address for hop in route[-2:0:-1]]},
                capabilities=SessionCapabilitiesBody(segmentation=True),
                use_response_buffer="5 MiB",
                target_port=int(wireguard_tunnel_port),
            ) as session:
                assert len(await swarm7[route[0]].api.session_list_clients(Protocol.UDP)) == 1

                logging.info(f"Test ready for execution @ listen port {session.listen_port}")

                # TODO: Placeholder for actual test
                await asyncio.sleep(3600)
