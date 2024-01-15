import fnmatch
import http.client
import json
import logging
import os
import random
import shutil
import subprocess
from copy import deepcopy
from pathlib import Path
from time import sleep

import pytest

from .hopr import HoprdAPI

random_data = os.urandom(8)
SEED = int.from_bytes(random_data, byteorder="big")
random.seed(SEED)

LOCALHOST = "127.0.0.1"
OPEN_CHANNEL_FUNDING_VALUE = "1000000000000000000000"

TICKET_AGGREGATION_THRESHOLD = 100
TICKET_PRICE_PER_HOP = 100

MYDIR = Path(__file__).parent
FIXTURE_FILES_DIR = Path("/tmp/")

FIXTURE_FILES_PREFIX = "hopr-smoke-test"
NODE_NAME_PREFIX = "hopr-smoke-test-node"

ANVIL_CFG_FILE = FIXTURE_FILES_DIR.joinpath(f"{FIXTURE_FILES_PREFIX}-anvil.cfg")
ANVIL_LOG_FILE = FIXTURE_FILES_DIR.joinpath(f"{FIXTURE_FILES_PREFIX}-anvil.log")
ANVIL_NETWORK1 = "anvil-localhost"
ANVIL_NETWORK2 = "anvil-localhost2"

PROTOCOL_CONFIG_FILE = MYDIR.parent.joinpath("scripts/protocol-config-anvil.json")
TEST_PROTOCOL_CONFIG_FILE = Path(f"/tmp/{NODE_NAME_PREFIX}-protocol-config.json")
DEPLOYMENTS_SUMMARY_FILE = MYDIR.parent.joinpath("ethereum/contracts/contracts-addresses.json")

PREGENERATED_IDENTITIES_DIR = MYDIR.joinpath("identities")

DEFAULT_API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"

def pytest_addoption(parser):
    parser.addoption(
        "--stress-seq-request-count",
        action="store",
        default=200,
        help="Number of sequential requests in the stress test",
    )
    parser.addoption(
        "--stress-par-request-count", action="store", default=200, help="Number of parallel requests in the stress test"
    )
    parser.addoption(
        "--stress-tested-api",
        action="store",
        default=f"http://{LOCALHOST}:{NODES['1'].api_port}",
        help="The API towards which the stress test is performed",
    )
    parser.addoption(
        "--stress-tested-api-token",
        action="store",
        default=DEFAULT_API_TOKEN,
        help="The token for the stress tested API",
    )
    parser.addoption(
        "--stress-minimum-peer-count", action="store", default=3, help="The minimum peer count to start the stress test"
    )


@pytest.fixture
def cmd_line_args(request):
    args = {
        "stress_seq_request_count": request.config.getoption("--stress-seq-request-count"),
        "stress_par_request_count": request.config.getoption("--stress-par-request-count"),
        "stress_tested_api": request.config.getoption("--stress-tested-api"),
        "stress_tested_api_token": request.config.getoption("--stress-tested-api-token"),
        "stress_minimum_peer_count": request.config.getoption("--stress-minimum-peer-count"),
    }

    return args


class Node:
    def __init__(self, api_port: int, p2p_prt: int, api_token: str, dir: Path, host_addr: str, cfg_file: Path = None, network: str = ANVIL_NETWORK1):
        self.api_port: int = api_port
        self.p2p_port: int = p2p_prt
        self.dir: Path = dir
        self.host_addr: str = host_addr
        self.api_token: str = api_token

        self.cfg_file: Path = cfg_file
        self.network: str = network
        
        self.peer_id: str = None
        self.address: str = None
        self.safe_address: str = None
        self.module_address: str = None
        self.proc = None
        self.api: HoprdAPI = None


NODES = {
    "1": Node(19091, 13301, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_1"), "localhost"),
    "2": Node(19092, 13302, None, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_2"), "127.0.0.1"),
    "3": Node(19093, 13303, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_3"), "localhost"),
    "4": Node(19094, 13304, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_4"), "127.0.0.1"),
    "5": Node(19095, 13305, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_5"), "localhost", FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_5.cfg.yaml")),
    "6": Node(19096, 13306, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_6"), "127.0.0.1", network=ANVIL_NETWORK2),
    "7": Node(19097, 13307, DEFAULT_API_TOKEN, FIXTURE_FILES_DIR.joinpath(f"{NODE_NAME_PREFIX}_7"), "localhost"),
}


def cleanup_node(node):
    try:
        node.proc.kill()
    except Exception:
        pass


def setup_node(node: Node):
    logging.info(f"Setting up a node with configuration: {node}")
    log_file = open(f"{node.dir}.log", "w")
    api_token_param = f"--api-token={node.api_token}" if node.api_token else "--disableApiAuthentication"
    custom_env = {
        "HOPRD_HEARTBEAT_INTERVAL": "2500",
        "HOPRD_HEARTBEAT_THRESHOLD": "2500",
        "HOPRD_HEARTBEAT_VARIANCE": "1000",
        "HOPRD_NETWORK_QUALITY_THRESHOLD": "0.3",
    }
    cmd = [
        "target/debug/hoprd",
        "--announce",
        "--api",
        "--disableTicketAutoRedeem",
        "--init",
        "--testAnnounceLocalAddresses",
        "--testPreferLocalAddresses",
        "--testUseWeakCrypto",
        f"--apiPort={node.api_port}",
        f"--data={node.dir}",
        f"--host={node.host_addr}:{node.p2p_port}",
        f"--identity={node.dir}.id",
        f"--moduleAddress={node.module_address}",
        f"--network={node.network}",
        f"--password={PASSWORD}",
        f"--safeAddress={node.safe_address}",
        f"--protocolConfig={TEST_PROTOCOL_CONFIG_FILE}",
        api_token_param,
    ]
    if node.cfg_file is not None:
        cmd += [f"--configurationFilePath={node.cfg_file}"]

    # remove previous databases
    shutil.rmtree(node.dir.as_posix(), ignore_errors=True)

    proc = subprocess.Popen(
        cmd, stdout=log_file, stderr=subprocess.STDOUT, env=os.environ | custom_env, cwd=MYDIR.parent.as_posix()
    )
    api = HoprdAPI(f"http://{node.host_addr}:{node.api_port}", node.api_token)

    return (proc, api)


def test_sanity():
    assert len(NODES) == len({n.api_port for n in NODES.values()}), "All API ports must be unique"
    assert len(NODES) == len({n.p2p_port for n in NODES.values()}), "All p2p ports must be unique"


def check_socket(address, port):
    import socket

    s = socket.socket()
    try:
        s.connect((address, port))
        return True
    except Exception:
        return False
    finally:
        s.close()


def mirror_contract_data(dest_file_path, src_file_path, src_network, dest_network):
    dest_data = ""

    with open(src_file_path, "r") as src_file, open(dest_file_path, "r") as dest_file:
        src_data = json.load(src_file)
        dest_data = json.load(dest_file)
        network_data = src_data["networks"][src_network]
        partial_network_data = {
            "environment_type": network_data["environment_type"],
            "indexer_start_block_number": 1,
            "addresses": network_data["addresses"],
        }
        new_network_data = dest_data["networks"][dest_network] | partial_network_data
        dest_data["networks"][dest_network] = new_network_data

    with open(dest_file_path, "w") as dest_file:
        json.dump(dest_data, dest_file, sort_keys=True, indent=2)


def reuse_pregenerated_identities():
    # remove existing identity files in tmp folder, .safe.args
    suffixes = [f"{FIXTURE_FILES_PREFIX}_*.safe.args", f"{FIXTURE_FILES_PREFIX}_*.id"]

    def is_relevant_file(f):
        return any([fnmatch.fnmatch(f, pattern) for pattern in suffixes])

    for f in filter(is_relevant_file, os.listdir(FIXTURE_FILES_DIR)):
        os.remove(FIXTURE_FILES_DIR.joinpath(f))
        logging.info(f"Removed file {FIXTURE_FILES_DIR.joinpath(f)}")

    # we copy and rename the files according to the expected file name format and destination folder
    node_nr = 1
    id_files = sorted(os.listdir(PREGENERATED_IDENTITIES_DIR))

    def is_relevant_file(f):
        return fnmatch.fnmatch(f, "*.id")

    for f in filter(lambda f: fnmatch.fnmatch(f, "*.id"), id_files):
        path = shutil.copyfile(
            PREGENERATED_IDENTITIES_DIR.joinpath(f), 
            FIXTURE_FILES_DIR.joinpath(f"{FIXTURE_FILES_PREFIX}-node_{node_nr}.id"),
        )
        logging.info(f"Copied file {PREGENERATED_IDENTITIES_DIR}/{f} to {path}")
        node_nr += 1


def create_local_safes(nodes_args: dict[str, Node], private_key):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "DEPLOYER_PRIVATE_KEY": private_key,
        "PRIVATE_KEY": private_key,
        "PATH": MYDIR.parent.joinpath(f".foundry/bin:{os.environ['PATH']}").as_posix(),
    }

    for node_id, node in nodes_args.items():
        id_file = f"{node.dir}.id"
        res = subprocess.run(
            [
                "hopli",
                "create-safe-module",
                "--network",
                ANVIL_NETWORK1,
                "--identity-from-path",
                id_file,
                "--contracts-root",
                "./ethereum/contracts",
                "--hopr-amount",
                "20000.0",
            ],
            env=os.environ | custom_env,
            check=True,
            capture_output=True,
            text=True,
        )
        for el in res.stdout.split("\n"):
            if el.startswith("safe: address 0x"):
                node.safe_address = el.split(" ")[-1]

            if el.startswith("module: address 0x"):
                node.module_address = el.split(" ")[-1]

    return nodes_args


def funding_nodes(private_key):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": MYDIR.parent.joinpath(f".foundry/bin:{os.environ['PATH']}").as_posix(),
    }
    subprocess.run(
        [
            "hopli",
            "faucet",
            "--network",
            ANVIL_NETWORK1,
            "--identity-prefix",
            FIXTURE_FILES_PREFIX,
            "--identity-directory",
            FIXTURE_FILES_DIR,
            "--contracts-root",
            "./ethereum/contracts",
            "--hopr-amount",
            "0.0",
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )


def collect_node_information(node):
    headers = {"Accept": "application/json"}
    if node.api_token is not None:
        headers = headers | {"x-auth-token": node.api_token}
    while True:
        try:
            conn = http.client.HTTPConnection(node.host_addr, node.api_port)
            conn.request("GET", "/api/v3/account/addresses", headers=headers)
            resp = conn.getresponse()
            assert resp.status == 200
            data = json.loads(resp.read())
            return (data["hopr"], data["native"])
        except Exception:
            sleep(0.1)


@pytest.fixture(scope="module")
def swarm7(request):
    logging.info(f"Using the random seed: {SEED}")

    for f in ["node_5.cfg.yaml"]:
        shutil.copyfile(MYDIR.joinpath(f), FIXTURE_FILES_DIR.joinpath(f"{FIXTURE_FILES_PREFIX}-{f}"))

    logging.info("Ensure local anvil server is not running")
    subprocess.run(["make", "kill-anvil"], cwd=MYDIR.parent, check=True)

    logging.info("Starting local anvil server")
    subprocess.run(
        ["./run-local-anvil.sh", "-l", ANVIL_LOG_FILE, "-c", ANVIL_CFG_FILE],
        check=True,
        capture_output=True,
        cwd=MYDIR.parent.joinpath("scripts"),
    )

    logging.info("Wait for anvil server to start up")

    # read auto-generated private key from anvil configuration
    with open(ANVIL_CFG_FILE, "r") as anvil_cfg_file:
        data = json.load(anvil_cfg_file)
        private_key = data["private_keys"][0]

    logging.info("Mirror contract data because of anvil-deploy node only writing to localhost")

    shutil.copyfile(PROTOCOL_CONFIG_FILE, TEST_PROTOCOL_CONFIG_FILE)
    mirror_contract_data(TEST_PROTOCOL_CONFIG_FILE, DEPLOYMENTS_SUMMARY_FILE, ANVIL_NETWORK1, ANVIL_NETWORK1)
    mirror_contract_data(TEST_PROTOCOL_CONFIG_FILE, DEPLOYMENTS_SUMMARY_FILE, ANVIL_NETWORK1, ANVIL_NETWORK2)

    logging.info("Reuse pre-generated identities")
    reuse_pregenerated_identities()

    nodes = deepcopy(NODES)

    logging.info("Create safe and modules for all the ids, store them in args files")
    nodes = create_local_safes(nodes, private_key)

    try:
        for node_id, node in nodes.items():
            (proc, api) = setup_node(node)
            node.proc = proc
            node.api: HoprdAPI = api

        logging.info(f"Wait for {len(nodes)} nodes to start up")

        for node_id, node in nodes.items():
            while True:
                with open(f"{node.dir}.log", "r") as f:
                    logs = f.read()
                    if "still unfunded, " in logs or "node is funded" in logs:
                        logging.info(f"Node {node_id} up")
                        break

                sleep(0.1)

        logging.info("Funding nodes")
        funding_nodes(private_key)

        logging.info("Collect node information")
        for node_id, node in nodes.items():
            peer_id, address = collect_node_information(node)
            node.peer_id = peer_id
            node.address = address

        logging.info("Node setup finished, waiting 10 seconds before proceeding")
        sleep(10)

        yield nodes
    except Exception as e:
        logging.error(f"Creating a 7 node cluster - FAILED: {e}")
    finally:
        logging.debug("Tearing down the 7 node cluster")
        for node_id, node in nodes.items():
            cleanup_node(node)
        pass
