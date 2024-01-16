import asyncio
import json
import logging
import os
import random
import shutil
import socket
from copy import deepcopy
from pathlib import Path
from subprocess import STDOUT, Popen, run

import pytest

from .node import Node

SEED = int.from_bytes(os.urandom(8), byteorder="big")
random.seed(SEED)

# Global variables
LOCALHOST = "127.0.0.1"
OPEN_CHANNEL_FUNDING_VALUE = 1000

TICKET_AGGREGATION_THRESHOLD = 100
TICKET_PRICE_PER_HOP = 100

FIXTURES_PREFIX = "hopr-smoke-test"
NODE_NAME_PREFIX = "hopr-smoke-test-node"

NETWORK1 = "anvil-localhost"
NETWORK2 = "anvil-localhost2"

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

NODES = [
    Node(
        19091,
        13301,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_1"),
        "localhost",
        NETWORK1,
    ),
    Node(
        19092,
        13302,
        None,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_2"),
        LOCALHOST,
        NETWORK1,
    ),
    Node(
        19093,
        13303,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_3"),
        "localhost",
        NETWORK1,
    ),
    Node(
        19094,
        13304,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_4"),
        LOCALHOST,
        NETWORK1,
    ),
    Node(
        19095,
        13305,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_5"),
        LOCALHOST,
        NETWORK1,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_5.cfg.yaml"),
    ),
    Node(
        19096,
        13306,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_6"),
        LOCALHOST,
        NETWORK2,
    ),
    Node(
        19097,
        13307,
        API_TOKEN,
        FIXTURES_DIR.joinpath(f"{NODE_NAME_PREFIX}_7"),
        "localhost",
        NETWORK1,
    ),
]


def pytest_addoption(parser: pytest.Parser):
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
        default=f"http://{LOCALHOST}:{NODES[0].api_port}",
        help="The API towards which the stress test is performed",
    )
    parser.addoption(
        "--stress-tested-api-token",
        action="store",
        default=API_TOKEN,
        help="The token for the stress tested API",
    )
    parser.addoption(
        "--stress-minimum-peer-count", action="store", default=3, help="The minimum peer count to start the stress test"
    )


@pytest.fixture
def cmd_line_args(request: pytest.FixtureRequest):
    args = {
        "stress_seq_request_count": request.config.getoption("--stress-seq-request-count"),
        "stress_par_request_count": request.config.getoption("--stress-par-request-count"),
        "stress_tested_api": request.config.getoption("--stress-tested-api"),
        "stress_tested_api_token": request.config.getoption("--stress-tested-api-token"),
        "stress_minimum_peer_count": request.config.getoption("--stress-minimum-peer-count"),
    }

    return args


def setup_node(node: Node):
    logging.info(f"Setting up {node}")
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

    with open(f"{node.dir}.log", "w") as log_file:
        node.proc = Popen(
            cmd,
            stdout=log_file,
            stderr=STDOUT,
            env=os.environ | custom_env,
            cwd=PWD.parent,
        )

    return node


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


def create_local_safe(node: Node, private_key: str):
    custom_env: dict = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "DEPLOYER_PRIVATE_KEY": private_key,
        "PRIVATE_KEY": private_key,
        "PATH": PWD.parent.joinpath(f".foundry/bin:{os.environ['PATH']}").as_posix(),
    }

    res = run(
        [
            "hopli",
            "create-safe-module",
            "--network",
            NETWORK1,
            "--identity-from-path",
            f"{node.dir}.id",
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
            node.safe_address = el.split()[-1]

        if el.startswith("module: address 0x"):
            node.module_address = el.split()[-1]

    return node


def fund_nodes(private_key: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": PWD.parent.joinpath(f".foundry/bin:{os.environ['PATH']}").as_posix(),
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

    ## STOP OLD LOCAL ANVIL SERVER
    logging.info("Ensure local anvil server is not running")
    run("make kill-anvil".split(), cwd=PWD.parent, check=True)

    ## START NEW LOCAL ANVIL SERVER
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

    nodes = deepcopy(NODES)

    nodes = [create_local_safe(node, private_key) for node in nodes]
    nodes = [setup_node(node) for node in nodes]

    # WAIT FOR NODES TO BE UP
    logging.info(f"Wait for {len(nodes)} nodes to start up")
    [await node.api.startedz() for node in nodes]

    # FUND NODES
    logging.info("Funding nodes")
    fund_nodes(private_key)

    # FINAL WAIT FOR NODES TO BE UP
    logging.info("Node setup finished, waiting for nodes to be up")
    for node in nodes:
        await node.api.readyz()
        node.peer_id = await node.api.addresses("hopr")
        node.address = await node.api.addresses("native")

    # YIELD NODES
    yield nodes

    # POST TEST CLEANUP
    logging.debug(f"Tearing down the {len(nodes)} nodes cluster")
    [node.clean_up() for node in nodes]
