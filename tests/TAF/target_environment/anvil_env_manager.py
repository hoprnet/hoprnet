import os
import binascii
import random
import socket
import string
import subprocess
from typing import List

from ..topologies.node import Node


class AnvilEnvironmentManager:
    HOST = "127.0.0.1"
    P2P_PORT_BASE = 19090
    API_PORT_BASE = 13300
    ANVIL_PORT = 8545

    DEBUG = "hopr*"
    NODE_ENV = "development"
    HOPRD_HEARTBEAT_INTERVAL = 2500
    HOPRD_HEARTBEAT_THRESHOLD = 2500
    HOPRD_HEARTBEAT_VARIANCE = 1000
    HOPRD_NETWORK_QUALITY_THRESHOLD = 0.3
    HOPRD_ON_CHAIN_CONFIRMATIONS = 2

    PASSWORD = "e2e-test"
    HOPRD_PASSWORD = "open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0"
    HOPRD_API_TOKEN = "^binary6wire6GLEEMAN9urbanebetween1watch^"
    DEFAULT_API_TOKEN = "e2e-API-token^^"

    tmp = "/tmp"
    password = "e2e-test"

    def __init__(self) -> None:
        self.env_vars = os.environ.copy()

    def __del__(self) -> None:
        self.cleanup()

    def setup_local_nodes(self, count) -> List[Node]:
        """
        Setting up a number of local nodes
        :count: The number of local nodes to setup
        """
        for port in range(1, count + 1):
            self.__ensure_port_is_free(port)
        self.__ensure_port_is_free(self.ANVIL_PORT)
        self.run_local_anvil()
        nodes = self.generate_nodes(count)

        self.__set_envar("DEBUG", self.DEBUG)
        self.__set_envar("NODE_ENV", self.NODE_ENV)
        self.__set_envar("HOPRD_HEARTBEAT_INTERVAL", str(self.HOPRD_HEARTBEAT_INTERVAL))
        self.__set_envar("HOPRD_HEARTBEAT_THRESHOLD", str(self.HOPRD_HEARTBEAT_THRESHOLD))
        self.__set_envar("HOPRD_HEARTBEAT_VARIANCE", str(self.HOPRD_HEARTBEAT_VARIANCE))
        self.__set_envar("HOPRD_NETWORK_QUALITY_THRESHOLD", str(self.HOPRD_NETWORK_QUALITY_THRESHOLD))
        self.__set_envar("HOPRD_ON_CHAIN_CONFIRMATIONS", str(self.HOPRD_ON_CHAIN_CONFIRMATIONS))

        for index, node in enumerate(nodes):
            self.setup_node(index, node)
        return nodes

    def run_local_anvil(self):
        """
        Starting the local anvil chain
        """
        if self.__port_in_use(self.ANVIL_PORT):
            print("Anvil chain already running, skipping...")
        else:
            print("Start local anvil chain...")
            self.anvil_process = subprocess.Popen(
                ["./.foundry/bin/anvil", "--host", self.HOST, "--block-time", "2", "--config-out", ".anvil.cfg"],
                shell=True,
                stdout=subprocess.PIPE,
            )
            while True:
                for line in self.anvil_process.stdout:
                    line_decoded = line.decode("utf-8")
                    if f"Listening on {self.HOST}:{self.ANVIL_PORT}" in line_decoded:
                        print(f"Anvil chain started successfully @({self.HOST}:{self.ANVIL_PORT})")
                        break
                else:
                    continue
                break

    def stop_local_anvil(self):
        """
        Stopping the execution of the local anvil process (for cleanup purposes)
        Prerequisite: The anvil process has to be already running
        """
        print("Kill local anvil chain...")
        if self.__port_in_use(self.ANVIL_PORT):
            try:
                self.anvil_process.kill()
                # sudo kill -9 `sudo lsof -t -i:8545`
            except AttributeError:
                pass

    def setup_node(self, nodeIndex: int, node: Node) -> None:
        """
        Setting up and running a node with given parameters.
        :nodeIndex: The index of the node.
        :node: The node configuration
        """
        print("node: " + str(node))
        dir = f"test-node-{nodeIndex}"
        if not os.path.isdir(dir):
            os.mkdir(dir)

        id = "".join([dir, ".id"])
        additional_args = "--testNoDirectConnections"
        additional_args = "--announce --network anvil-localhost2"
        additional_args = "--announce"
        "".join([dir, ".log"])
        command = [
            "node",
            "--experimental-wasm-modules",
            "--experimental-wasm-reftypes",
            "packages/hoprd/lib/main.cjs",
            f'--data="{dir}"',
            f"--host={self.HOST}:{node.p2p_port}",
            f'--identity="{id}"',
            "--init",
            f'--password="{self.password}"',
            f'--privateKey="{node.private_key}"',
            "--api",
            "--apiPort",
            f"{node.api_port}",
            "--testAnnounceLocalAddresses",
            "--testPreferLocalAddresses",
            "--testUseWeakCrypto",
            "--allowLocalNodeConnections",
            "--allowPrivateNodeConnections",
            f"{additional_args}"
            # f'> "{log}" 2>&1 &'
        ]
        print(command)
        self.node_process = subprocess.run(command)

    def generate_nodes(self, count) -> List[Node]:
        """
        Generating a list of nodes.
        :count: The number of nodes to generate
        """
        nodes = []
        for i in range(count):
            nodes.append(self.generate_node(i + 1))
        return nodes

    def generate_node(self, index) -> Node:
        """
        Generating a node
        :index: The node index to be used to building up the p2p and api ports
        :return: A node object
        """
        node: Node = Node(
            p2p_port=self.P2P_PORT_BASE + index,
            api_port=self.API_PORT_BASE + index,
            private_key=self.generate_private_key(),
            peer_id=self.generate_peer_id(),
        )
        return node

    def register_node(self) -> None:
        """
        Registering a node within the network.
        """
        pass

    def generate_private_key(self) -> str:
        """
        Generate 64-digit hexadecimal number.
        Example: 0xb0c01844be5d5deaf7514592be16594a05522ada5c0e22a24aa8a7ec7bc180b2
        :privateKey: The 64-byte random generated private key
        """
        hex: bytes = binascii.b2a_hex(os.urandom(32))
        privateKey: str = "0x" + hex.decode("utf-8")
        return privateKey

    def generate_peer_id(self) -> str:
        """
        Generate 53-digit peer id with random lower and capital case ascii letters and digits, starting with '16U'
        Example: 16ULoTSioYpLX0Bwqh1LD4YAsm2uSnwgyhSKMY9RRXNKwzKIsXKBv
        :return: 53-digit peer id starting with '16U'
        """
        string50 = "".join(random.choices(string.ascii_letters + string.digits, k=50))
        peerId = "16U" + string50
        return peerId

    def cleanup(self) -> None:
        """
        Cleanup node directories, etc
        """
        pass

    def __ensure_port_is_free(self, port):
        """ """
        if self.__port_in_use(port):
            raise Exception(f"Port {port} is in use")

    def __port_in_use(self, port):
        """
        Check if a given port is in use.
        :port: The port to check against localhost.
        """
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        result = sock.connect_ex((self.HOST, port))
        if result == 0:
            return True
        else:
            return False

    def __set_envar(self, name: str, value: str):
        """
        Generic helper method used to set an environment variable within the OS
        """
        os.environ.setdefault(name, value)


if __name__ == "__main__":
    env = AnvilEnvironmentManager()
    # print(env.generate_private_key())
    # print(env.generate_peer_id())
    # setup_nodes = env.generate_nodes(1)
    # nodes_api_as_str = " ".join(list(map(lambda x: f"\"localhost:{x['api_port']}\"", setup_nodes.values())))
    # print(nodes_api_as_str)
    env.setup_local_nodes(5)
