import asyncio
import itertools
import json
import logging
import os
import random
import shutil
import socket
from copy import deepcopy
from pathlib import Path
from subprocess import run

import pytest

from .node import Node

# prepend the timestamp in front of any log line
logging.basicConfig(format="%(asctime)s %(message)s")

SEED = int.from_bytes(os.urandom(8), byteorder="big")
random.seed(SEED)


def pytest_addoption(parser: pytest.Parser):
    parser.addoption(
        "--stress-request-count",
        action="store",
        default=500,
        help="Number of requests performed by each source node towards the stressed target",
    )
    parser.addoption(
        "--stress-sources",
        action="store",
        type=str,
        help="The JSON string containing the list of dicts with 'url' and 'token' keys for each stress test source node",
    )
    parser.addoption(
        "--stress-target",
        action="store",
        type=str,
        help="The JSON string containing the dict with 'url' and 'token' keys for the stressed target node",
    )


# Global variables
LOCALHOST = "127.0.0.1"
OPEN_CHANNEL_FUNDING_VALUE_HOPR = 1000

TICKET_AGGREGATION_THRESHOLD = 100
TICKET_PRICE_PER_HOP = 100

RESERVED_TAG_UPPER_BOUND = 1023


FIXTURES_PREFIX = "hopr-smoke-test"
NODE_NAME_PREFIX = f"{FIXTURES_PREFIX}-node"

NETWORK1 = "anvil-localhost"
NETWORK2 = "anvil-localhost2"
ANVIL_ENDPOINT = "localhost:8545"

API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"

PWD = Path(__file__).parent
FIXTURES_DIR = Path("/tmp")

ANVIL_CFG_FILE = FIXTURES_DIR.joinpath(f"{FIXTURES_PREFIX}-anvil.cfg")
ANVIL_LOG_FILE = FIXTURES_DIR.joinpath(f"{FIXTURES_PREFIX}-anvil.log")

PROTOCOL_CONFIG_FILE = PWD.parent.joinpath("scripts/protocol-config-anvil.json")
TEST_PROTOCOL_CONFIG_FILE = FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}-protocol-config.json")
DEPLOYMENTS_SUMMARY_FILE = PWD.parent.joinpath("ethereum/contracts/contracts-addresses.json")
PREGENERATED_IDENTITIES_DIR = PWD.joinpath("identities")

NODES = {
    "1": Node(
        19091,
        13301,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_1"),
        "localhost",
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
    "2": Node(
        19092,
        13302,
        None,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_2"),
        LOCALHOST,
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
    "3": Node(
        19093,
        13303,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_3"),
        "localhost",
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
    "4": Node(
        19094,
        13304,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_4"),
        LOCALHOST,
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
    "5": Node(
        19095,
        13305,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_5"),
        "localhost",
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_default.cfg.yaml"),
    ),
    "6": Node(
        19096,
        13306,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_6"),
        LOCALHOST,
        NETWORK2,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
    "7": Node(
        19097,
        13307,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_7"),
        "localhost",
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_barebone.cfg.yaml"),
    ),
}


def barebone_nodes():
    """Nodes using only the barebones config without any strategies"""
    return ["1", "2", "3", "4"]


def nodes_with_auth():
    """All nodes within the same network as specified in the swarm7 fixture"""
    return ["1", "3", "4"]


def default_nodes():
    """A node that uses the default strategies"""
    return ["5"]


def random_distinct_pairs_from(values: list, count: int):
    return random.sample([(left, right) for left, right in itertools.product(values, repeat=2) if left != right], count)


def check_socket(address: str, port: str):
    s = socket.socket()
    try:
        s.connect((address, port))
        return True
    except Exception:
        return False
    finally:
        s.close()


def mirror_contract_data(dest_file_path: Path, src_file_path: Path, src_network: str, dest_network: str):
    with open(src_file_path, "r") as file:
        src_data = json.load(file)

    with open(dest_file_path, "r") as file:
        dest_data = json.load(file)

    network_data = src_data["networks"][src_network]
    partial_network_data = {
        "environment_type": network_data["environment_type"],
        "indexer_start_block_number": 1,
        "addresses": network_data["addresses"],
    }
    new_network_data = dest_data["networks"][dest_network] | partial_network_data
    dest_data["networks"][dest_network] = new_network_data

    with open(dest_file_path, "w") as file:
        json.dump(dest_data, file, sort_keys=True)


def copy_identities():
    # Remove old identities
    for f in FIXTURES_DIR.glob(f"{FIXTURES_PREFIX}-*.id"):
        os.remove(f)
    logging.info(f"Removed '*.id' files in {FIXTURES_DIR}")

    # Remove old logs
    for f in FIXTURES_DIR.glob(f"{FIXTURES_PREFIX}-*.log"):
        os.remove(f)
    logging.info(f"Removed '*.log' files in {FIXTURES_DIR}")

    # Remove old db
    for f in FIXTURES_DIR.glob(f"{FIXTURES_PREFIX}-*"):
        if not f.is_dir():
            continue
        shutil.rmtree(f, ignore_errors=True)
    logging.info(f"Removed dbs in {FIXTURES_DIR}")

    # Copy new identity files
    for node_id in range(len(NODES)):
        f = f"{NODE_NAME_PREFIX}_{node_id+1}.id"
        shutil.copy(
            PREGENERATED_IDENTITIES_DIR.joinpath(f),
            FIXTURES_DIR.joinpath(f),
        )
    logging.info(f"Copied '*.id' files to {FIXTURES_DIR}")

    # Copy new config files
    for f in PWD.glob(f"{NODE_NAME_PREFIX}_*.cfg.yaml"):
        shutil.copy(f, FIXTURES_DIR.joinpath(f.name))
    logging.info(f"Copied '*.cfg.yaml' files to {FIXTURES_DIR}")


def fund_nodes(private_key: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "faucet",
            "--network",
            NETWORK1,
            "--identity-prefix",
            FIXTURES_PREFIX,
            "--identity-directory",
            FIXTURES_DIR,
            "--contracts-root",
            "./ethereum/contracts",
            "--hopr-amount",
            "0.0",
            "--native-amount",
            "10.0",
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )


@pytest.fixture(scope="module")
def event_loop():
    policy = asyncio.get_event_loop_policy()
    loop = policy.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(scope="module")
async def swarm7(request):
    logging.info(f"Using the random seed: {SEED}")

    # STOP OLD LOCAL ANVIL SERVER
    logging.info("Ensure local anvil server is not running")
    run(["make", "kill-anvil"], cwd=PWD.parent, check=True)

    # START NEW LOCAL ANVIL SERVER
    logging.info("Starting and waiting for local anvil server to be up")
    run(
        f"./run-local-anvil.sh -l {ANVIL_LOG_FILE} -c {ANVIL_CFG_FILE}".split(),
        check=True,
        capture_output=True,
        cwd=PWD.parent.joinpath("scripts"),
    )

    # READ AUTO_GENERATED PRIVATE-KEY FROM ANVIL CONFIGURATION
    with open(ANVIL_CFG_FILE, "r") as file:
        data: dict = json.load(file)
        private_key = data.get("private_keys", [""])[0]

    logging.info("Mirror contract data because of anvil-deploy node only writing to localhost")

    shutil.copy(PROTOCOL_CONFIG_FILE, TEST_PROTOCOL_CONFIG_FILE)
    mirror_contract_data(TEST_PROTOCOL_CONFIG_FILE, DEPLOYMENTS_SUMMARY_FILE, NETWORK1, NETWORK1)
    mirror_contract_data(TEST_PROTOCOL_CONFIG_FILE, DEPLOYMENTS_SUMMARY_FILE, NETWORK1, NETWORK2)

    # SETUP NODES USING STORED IDENTITIES
    logging.info("Reuse pre-generated identities and configs")
    copy_identities()

    # CREATE LOCAL SAFES AND MODULES FOR ALL THE IDS
    logging.info("Create safe and modules for all the ids, store them in args files")

    nodes: dict[str, Node] = deepcopy(NODES)

    safe_custom_env: dict = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }

    for node in nodes.values():
        logging.info(f"Creating safe and module for {node}")
        node.create_local_safe(safe_custom_env)

    # wait before contract deployments are finalized
    await asyncio.sleep(5)

    for node in nodes.values():
        logging.info(f"Setting up {node}")
        node.setup(PASSWORD, TEST_PROTOCOL_CONFIG_FILE, PWD.parent)

    # WAIT FOR NODES TO BE UP
    logging.info(f"Wait for {len(nodes)} nodes to start up")

    # minimal wait to ensure api is ready for `startedz` call.
    for id, node in nodes.items():
        await asyncio.wait_for(node.api.startedz(), timeout=60)
        logging.info(f"Node {id} is up")

    # FUND NODES
    logging.info("Funding nodes")
    fund_nodes(private_key)

    # WAIT FOR NODES TO BE UP
    logging.info("Node setup finished, waiting for nodes to be ready")
    for node in nodes.values():
        while not await asyncio.wait_for(node.api.readyz(), timeout=60):
            logging.info(f"Node {node} not ready yet, retrying")
            await asyncio.sleep(1)

        addresses = await node.api.addresses()
        node.peer_id = addresses["hopr"]
        node.address = addresses["native"]
        logging.info(f"Node {node} is ready")

    # WAIT FOR NODES TO CONNECT TO ALL PEERS
    logging.info("Waiting for nodes to connect to all peers")
    for node in nodes.values():

        def is_same_node(a, b):
            return a.peer_id == b.peer_id

        def is_in_same_network(a, b):
            return a.network == b.network

        required_peers = [
            n.peer_id for n in nodes.values() if not is_same_node(n, node) and is_in_same_network(n, node)
        ]

        async def all_peers_connected():
            peers = [p["peer_id"] for p in await asyncio.wait_for(node.api.peers(), timeout=60)]
            missing_peers = [p for p in required_peers if p not in peers]
            return len(missing_peers) == 0

        while not await all_peers_connected():
            logging.info(f"Node {node} does not have all peers connected yet, retrying")
            await asyncio.sleep(1)

    # YIELD NODES
    yield nodes

    # POST TEST CLEANUP
    logging.debug(f"Tearing down the {len(nodes)} nodes cluster")
    [node.clean_up() for node in nodes.values()]


def to_ws_url(host, port):
    return f"ws://{host}:{port}/api/v3/messages/websocket"
