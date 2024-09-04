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
        default=1000,
        type=int,
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

FIXTURES_PREFIX = "hopr"
NODE_NAME_PREFIX = f"{FIXTURES_PREFIX}-node"

NETWORK1 = "anvil-localhost"
NETWORK2 = "anvil-localhost2"

API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"

# the test framework uses the following folder structure:
#
# /tmp/hopr-smoke-test/ - parent directory for all test related files
# /tmp/hopr-smoke-test/snapshot/ - directory for snapshot files which can be re-used
# /tmp/hopr-smoke-test/${SUITE_NAME}/ - directory for a specific test suite
# /tmp/hopr-smoke-test/${SUITE_NAME}/hopr-node_*.id - identity file for nodes
# /tmp/hopr-smoke-test/${SUITE_NAME}/hopr-node_*.log - log file for nodes
# /tmp/hopr-smoke-test/${SUITE_NAME}/anvil.cfg - anvil configuration file
# /tmp/hopr-smoke-test/${SUITE_NAME}/anvil.log - anvil configuration file

PWD = Path(__file__).parent

def fixtures_dir(name: str): return Path(f"/tmp/hopr-smoke-test/{name}")
def anvil_cfg_file(name: str): return Path(f"{fixtures_dir(name)}/anvil.cfg")
def anvil_log_file(name: str): return Path(f"{fixtures_dir(name)}/anvil.log")
def protocol_config_file(name: str): return Path(f"{fixtures_dir(name)}/protocol-config.json")
def snapshot_dir(parent_dir: Path): return parent_dir.joinpath("snapshot")
def anvil_state_file(parent_dir: Path): return parent_dir.joinpath("anvil.state.json")

INPUT_PROTOCOL_CONFIG_FILE = PWD.parent.joinpath("scripts/protocol-config-anvil.json")
INPUT_DEPLOYMENTS_SUMMARY_FILE = PWD.parent.joinpath("ethereum/contracts/contracts-addresses.json")
PREGENERATED_IDENTITIES_DIR = PWD.joinpath("identities")

NODES = {
    "1": Node(
        1,
        API_TOKEN,
        "localhost",
        NETWORK1,
        "barebone.cfg.yaml",
    ),
    "2": Node(
        2,
        None,
        LOCALHOST,
        NETWORK1,
        "barebone.cfg.yaml",
    ),
    "3": Node(
        3,
        API_TOKEN,
        "localhost",
        NETWORK1,
        "barebone.cfg.yaml",
    ),
    "4": Node(
        4,
        API_TOKEN,
        LOCALHOST,
        NETWORK1,
        "barebone.cfg.yaml",
    ),
    "5": Node(
        5,
        API_TOKEN,
        "localhost",
        NETWORK1,
        "default.cfg.yaml",
    ),
    "6": Node(
        6,
        API_TOKEN,
        LOCALHOST,
        NETWORK2,
        "barebone.cfg.yaml",
    ),
    "7": Node(
        7,
        API_TOKEN,
        "localhost",
        NETWORK1,
        "barebone.cfg.yaml",
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


def nodes_with_different_network():
    """Nodes with different network"""
    return ["6", "7"]


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


def cleanup_data(parent_dir: Path):
    # Remove old db
    for f in parent_dir.glob(f"{NODE_NAME_PREFIX}_*"):
        if not f.is_dir():
            continue
        logging.info(f"Remove db in {f}")
        shutil.rmtree(f, ignore_errors=True)
    logging.info(f"Removed all dbs in {parent_dir}")

def copy_identities(dir: Path):
    # Remove old identities
    for f in dir.glob(f"{NODE_NAME_PREFIX}*.id"):
        os.remove(f)
    logging.info(f"Removed '*.id' files in {dir}")

    # Remove old logs
    for f in dir.glob(f"{NODE_NAME_PREFIX}_*.log"):
        os.remove(f)
    logging.info(f"Removed '*.log' files in {dir}")

    # Copy new identity files
    for node_id in range(len(NODES)):
        f = f"{NODE_NAME_PREFIX}_{node_id+1}.id"
        shutil.copy(
            PREGENERATED_IDENTITIES_DIR.joinpath(f),
            dir.joinpath(f),
        )
    logging.info(f"Copied '*.id' files to {dir}")

    # Copy new config files
    for f in PWD.glob("*.cfg.yaml"):
        shutil.copy(f, dir.joinpath(f.name))
    logging.info(f"Copied '*.cfg.yaml' files to {dir}")


def snapshot_reuse(parent_dir: Path, nodes):
    sdir = snapshot_dir(parent_dir)

    # copy anvil state
    parent_dir.joinpath("anvil.state.json").unlink(missing_ok=True)
    shutil.copy(sdir.joinpath("anvil.state.json"), parent_dir)

    # copy configuration files
    for f in sdir.glob("*.cfg.yaml"):
        parent_dir.joinpath(f.name).unlink(missing_ok=True)
        shutil.copy(f, parent_dir)

    # copy node data
    for i in range(len(nodes)):
        node_target_dir = parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}/db/")
        node_snapshot_dir = sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}/db/")

        shutil.rmtree(node_target_dir, ignore_errors=True)
        node_target_dir.mkdir(parents=True, exist_ok=False)

        shutil.copy(node_snapshot_dir.joinpath("hopr_index.db"), node_target_dir)
        shutil.copy(node_snapshot_dir.joinpath("hopr_index.db-shm"), node_target_dir)
        shutil.copy(node_snapshot_dir.joinpath("hopr_index.db-wal"), node_target_dir)

        parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}.env").unlink(missing_ok=True)
        shutil.copy(sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}.env"), parent_dir)

def snapshot_create(anvil_port, parent_dir: Path, nodes):
    sdir = snapshot_dir(parent_dir)

    # delete old snapshot
    shutil.rmtree(sdir, ignore_errors=True)

    # create new snapshot
    sdir.mkdir(parents=True, exist_ok=True)

    # stop anvil and nodes
    [node.clean_up() for node in nodes.values()]
    run(["make", "kill-anvil", f"port={anvil_port}"], cwd=PWD.parent, check=True)

    # copy anvil state
    shutil.copy(anvil_state_file(parent_dir), sdir)

    # copy configuration files
    for f in parent_dir.glob("*.cfg.yaml"):
        shutil.copy(f, sdir)

    # copy node data and env files
    for i in range(len(nodes)):
        node_dir = parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
        node_target_dir = sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}/db/")
        node_target_dir.mkdir(parents=True, exist_ok=True)

        shutil.copy(f"{node_dir}/db/hopr_index.db", node_target_dir)
        shutil.copy(f"{node_dir}/db/hopr_index.db-shm", node_target_dir)
        shutil.copy(f"{node_dir}/db/hopr_index.db-wal", node_target_dir)
        shutil.copy(f"{node_dir}.env", sdir)


def snapshot_usable(parent_dir: Path, nodes):
    sdir = snapshot_dir(parent_dir)

    expected_files = [
        "anvil.state.json",
        "barebone.cfg.yaml",
        "default.cfg.yaml",
    ]
    for i in range(len(nodes)):
        node_dir = parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
        expected_files.append(f"{node_dir}/db/hopr_index.db")
        expected_files.append(f"{node_dir}/db/hopr_index.db-shm")
        expected_files.append(f"{node_dir}/db/hopr_index.db-wal")
        expected_files.append(f"{node_dir}.env")

    return all([sdir.joinpath(f).exists() for f in expected_files])

def fund_nodes(test_suite_name, test_dir: Path, anvil_port):
    private_key = load_private_key(test_suite_name)

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
            test_dir,
            "--contracts-root",
            "./ethereum/contracts",
            "--hopr-amount",
            "0.0",
            "--native-amount",
            "10.0",
            "--provider-url",
            f"http://127.0.0.1:{anvil_port}",
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

async def shared_nodes_bringup(test_suite_name: str, test_dir: Path, anvil_port,
                               nodes, skip_funding=False):
    for node in nodes.values():
        logging.info(f"Setting up {node}")
        node.setup(PASSWORD, protocol_config_file(test_suite_name), PWD.parent)

    # WAIT FOR NODES TO BE UP
    logging.info(f"Wait for {len(nodes)} nodes to start up")

    # minimal wait to ensure api is ready for `startedz` call.
    for id, node in nodes.items():
        await asyncio.wait_for(node.api.startedz(), timeout=60)
        logging.info(f"Node {id} is up")

    if not skip_funding:
      # FUND NODES
      logging.info("Funding nodes")
      fund_nodes(test_suite_name, test_dir, anvil_port)

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

def load_private_key(test_suite_name, pos = 0):
    with open(anvil_cfg_file(test_suite_name), "r") as file:
        data: dict = json.load(file)
        return data.get("private_keys", [""])[pos]


@pytest.fixture(scope="module")
def event_loop():
    policy = asyncio.get_event_loop_policy()
    loop = policy.new_event_loop()
    yield loop
    loop.close()

@pytest.fixture(scope="module")
async def paths(request):
    test_suite = request.module
    test_suite_name = test_suite.__name__.split('.')[-1]

    paths = {
        anvil_cfg_file: anvil_cfg_file(test_suite_name),
    }

    yield paths

@pytest.fixture(scope="module")
async def swarm7(request):
    logging.info(f"Using the random seed: {SEED}")

    # PREPARE TEST SUITE ENVIRONMENT
    test_suite = request.module
    test_suite_name = test_suite.__name__.split('.')[-1]
    if test_suite.PORT_BASE is None:
        raise ValueError("PORT_BASE must be set in the test suite")
    test_dir = fixtures_dir(test_suite_name)
    test_dir.mkdir(parents=True, exist_ok=True)
    anvil_port = test_suite.PORT_BASE
    logging.info(f"Setting test suite {test_suite_name} up: test_dir={test_dir}, anvil_port={anvil_port}")

    nodes: dict[str, Node] = deepcopy(NODES)
    for node in nodes.values():
        node.prepare(test_suite.PORT_BASE, test_dir, NODE_NAME_PREFIX)

    # STOP OLD LOCAL ANVIL SERVER
    logging.info("Ensure local anvil server is not running")
    run(["make", "kill-anvil", f"port={anvil_port}"], cwd=PWD.parent, check=True)

    use_snapshot = snapshot_usable(test_dir, nodes)

    cleanup_data(test_dir)

    if not use_snapshot:
        logging.info("Snapshot not usable")

        # START NEW LOCAL ANVIL SERVER
        logging.info("Starting and waiting for local anvil server to be up (dump state enabled)")
        run(
            f"./run-local-anvil.sh -l {anvil_log_file(test_suite_name)} -c {anvil_cfg_file(test_suite_name)} -p {anvil_port} -ds {anvil_state_file(test_dir)}"
            .split(),
            check=True,
            capture_output=True,
            cwd=PWD.parent.joinpath("scripts"),
        )

        logging.info("Mirror contract data because of anvil-deploy node only writing to localhost")

        shutil.copy(INPUT_PROTOCOL_CONFIG_FILE, protocol_config_file(test_suite_name))
        mirror_contract_data(protocol_config_file(test_suite_name), INPUT_DEPLOYMENTS_SUMMARY_FILE, NETWORK1, NETWORK1)
        mirror_contract_data(protocol_config_file(test_suite_name), INPUT_DEPLOYMENTS_SUMMARY_FILE, NETWORK1, NETWORK2)

        # SETUP NODES USING STORED IDENTITIES
        logging.info("Reuse pre-generated identities and configs")
        copy_identities(test_dir)

        # CREATE LOCAL SAFES AND MODULES FOR ALL THE IDS
        logging.info("Create safe and modules for all the ids, store them in args files")

        private_key = load_private_key(test_suite_name)

        safe_custom_env: dict = {
            "ETHERSCAN_API_KEY": "anykey",
            "IDENTITY_PASSWORD": PASSWORD,
            "MANAGER_PRIVATE_KEY": private_key,
            "PRIVATE_KEY": private_key,
            "PATH": os.environ["PATH"],
        }

        for node in nodes.values():
            logging.info(f"Creating safe and module for {node}")
            assert node.create_local_safe(safe_custom_env)

        # wait before contract deployments are finalized
        await asyncio.sleep(5)

        # BRING UP NODES (with funding)
        await shared_nodes_bringup(test_suite_name, test_dir, anvil_port, nodes)

        logging.info("Taking snapshot")
        snapshot_create(anvil_port, test_dir, nodes)

    logging.info("Re-using snapshot")
    snapshot_reuse(test_dir, nodes)

    logging.info("Starting and waiting for local anvil server to be up (load state enabled)")
    run(
        f"./run-local-anvil.sh -s -l {anvil_log_file(test_suite_name)} -c {anvil_cfg_file(test_suite_name)} -p {anvil_port} -ls {anvil_state_file(test_dir)}".split(),
        check=True,
        capture_output=True,
        cwd=PWD.parent.joinpath("scripts"),
    )

    # SETUP NODES USING STORED IDENTITIES
    logging.info("Reuse pre-generated identities and configs")
    copy_identities(test_dir)
    for node in nodes.values():
        node.load_addresses()

    # BRING UP NODES (without funding)
    await shared_nodes_bringup(test_suite_name, test_dir, anvil_port, nodes, skip_funding=True)

    # YIELD NODES
    logging.info("Nodes all ready, starting test")
    yield nodes

    # POST TEST CLEANUP
    logging.info(f"Tearing down the {len(nodes)} nodes cluster")
    [node.clean_up() for node in nodes.values()]
    run(["make", "kill-anvil", f"port={anvil_port}"], cwd=PWD.parent, check=True)


def to_ws_url(host, port):
    return f"ws://{host}:{port}/api/v3/messages/websocket"
