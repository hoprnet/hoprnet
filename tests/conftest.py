import fnmatch
import http.client
import itertools
import json
import logging
import os
import random
import shutil
import subprocess
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


def pytest_addoption(parser):
    parser.addoption(
        "--stress-seq-request-count",
        action="store",
        default=500,
        help="Number of sequential requests in the stress test",
    )
    parser.addoption(
        "--stress-par-request-count", action="store", default=200, help="Number of parallel requests in the stress test"
    )
    parser.addoption(
        "--stress-tested-api",
        action="store",
        default=f"http://{LOCALHOST}:{NODES['1']['api_port']}",
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


MYDIR = os.path.dirname(os.path.realpath(__file__))

FIXTURE_FILES_DIR = "/tmp/"
FIXTURE_FILES_PREFIX = "hopr-smoke-test"

NODE_NAME_PREFIX = "hopr-smoke-test-node"

ANVIL_CFG_FILE = f"{FIXTURE_FILES_DIR}{FIXTURE_FILES_PREFIX}-anvil.cfg"
ANVIL_LOG_FILE = f"{FIXTURE_FILES_DIR}{FIXTURE_FILES_PREFIX}-anvil.log"
ANVIL_NETWORK1 = "anvil-localhost"
ANVIL_NETWORK2 = "anvil-localhost2"

PROTOCOL_CONFIG_FILE = f"{MYDIR}/../scripts/protocol-config-anvil.json"
TEST_PROTOCOL_CONFIG_FILE = f"/tmp/{NODE_NAME_PREFIX}-protocol-config.json"
DEPLOYMENTS_SUMMARY_FILE = f"{MYDIR}/../ethereum/contracts/contracts-addresses.json"

PREGENERATED_IDENTITIES_DIR = f"{MYDIR}/identities"

DEFAULT_API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"
NODES = {
    "1": {
        "api_port": 19091,
        "p2p_port": 13301,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_1",
        "host_addr": "localhost",
    },
    "2": {
        "api_port": 19092,
        "p2p_port": 13302,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_2",
        "host_addr": "127.0.0.1",
    },
    "3": {
        "api_port": 19093,
        "p2p_port": 13303,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_3",
        "host_addr": "localhost",
    },
    "4": {
        "api_port": 19094,
        "p2p_port": 13304,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_4",
        "host_addr": "127.0.0.1",
    },
    "5": {
        "api_port": 19095,
        "p2p_port": 13305,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_5",
        "host_addr": "localhost",
        "cfg_file": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_5.cfg.yaml",
    },
    "6": {
        "api_port": 19096,
        "p2p_port": 13306,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_6",
        "host_addr": "127.0.0.1",
        "network": ANVIL_NETWORK2,
    },
    "7": {
        "api_port": 19097,
        "p2p_port": 13307,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_7",
        "host_addr": "localhost",
    },
}


def default_nodes():
    """All nodes within the same network as specified in the swarm7 fixture"""
    return list(NODES.keys())[:4]


def default_nodes_with_auth():
    """All nodes within the same network as specified in the swarm7 fixture"""
    return ["1", "3", "4"]


def random_distinct_pairs_from(values: list, count: int):
    return random.sample([(left, right) for left, right in itertools.product(values, repeat=2) if left != right], count)


def cleanup_node(args):
    try:
        proc = args["proc"]
        proc.kill()
    except Exception:
        pass


def setup_node(args: dict):
    logging.info(f"Setting up a node with configuration: {args}")
    log_file = open(f"{args['dir']}.log", "w")
    network = args.get("network", ANVIL_NETWORK1)
    api_token = args.get("api_token", None)
    api_token_param = f"--api-token={api_token}" if api_token else "--disableApiAuthentication"
    custom_env = {
        "HOPRD_HEARTBEAT_INTERVAL": "2500",
        "HOPRD_HEARTBEAT_THRESHOLD": "2500",
        "HOPRD_HEARTBEAT_VARIANCE": "1000",
        "HOPRD_NETWORK_QUALITY_THRESHOLD": "0.3",
    }
    cmd = [
        "hoprd",
        "--announce",
        "--api",
        "--disableTicketAutoRedeem",
        "--init",
        "--testAnnounceLocalAddresses",
        "--testPreferLocalAddresses",
        f"--apiPort={args['api_port']}",
        f"--data={args['dir']}",
        f"--host={args['host_addr']}:{args['p2p_port']}",
        f"--identity={args['dir']}.id",
        f"--moduleAddress={args['module_address']}",
        f"--network={network}",
        f"--password={PASSWORD}",
        f"--safeAddress={args['safe_address']}",
        f"--protocolConfig={TEST_PROTOCOL_CONFIG_FILE}",
        api_token_param,
    ]
    if "cfg_file" in args:
        cmd = cmd + [f"--configurationFilePath={args['cfg_file']}"]

    # remove previous databases
    shutil.rmtree(args["dir"], ignore_errors=True)

    proc = subprocess.Popen(
        cmd, stdout=log_file, stderr=subprocess.STDOUT, env=os.environ | custom_env, cwd=f"{MYDIR}/../"
    )
    api = HoprdAPI(f"http://{args['host_addr']}:{args['api_port']}", api_token)

    return (proc, api)


def test_sanity():
    assert len(NODES.keys()) == len(set([i["api_port"] for i in NODES.values()])), "All API ports must be unique"

    assert len(NODES.keys()) == len(set([i["p2p_port"] for i in NODES.values()])), "All p2p ports must be unique"


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

    def is_relevant_fixture_file(f):
        return any([fnmatch.fnmatch(f, pattern) for pattern in suffixes])

    for f in filter(is_relevant_fixture_file, os.listdir(FIXTURE_FILES_DIR)):
        os.remove(f"{FIXTURE_FILES_DIR}{f}")
        logging.info(f"Removed file {FIXTURE_FILES_DIR}{f}")

    # we copy and rename the files according to the expected file name format and destination folder
    node_nr = 1
    id_files = sorted(os.listdir(PREGENERATED_IDENTITIES_DIR))

    def is_relevant_id_file(f):
        return fnmatch.fnmatch(f, "*.id")

    for f in filter(is_relevant_id_file, id_files):
        shutil.copyfile(
            f"{PREGENERATED_IDENTITIES_DIR}/{f}", f"{FIXTURE_FILES_DIR}{FIXTURE_FILES_PREFIX}-node_{node_nr}.id"
        )
        node_nr += 1


def create_local_safes(nodes_args: dict, private_key):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "DEPLOYER_PRIVATE_KEY": private_key,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    for node_id, node_args in nodes_args.items():
        id_file = f"{node_args['dir']}.id"
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
                [_, _, addr] = el.split(" ")
                node_args["safe_address"] = addr

            if el.startswith("module: address 0x"):
                [_, _, addr] = el.split(" ")
                node_args["module_address"] = addr

        nodes_args[node_id] = node_args

    return nodes_args


def funding_nodes(private_key):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
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


def collect_node_information(node_args):
    headers = {"Accept": "application/json"}
    if "api_token" in node_args:
        headers = headers | {"x-auth-token": node_args["api_token"]}
    while True:
        try:
            conn = http.client.HTTPConnection(node_args["host_addr"], node_args["api_port"])
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
        shutil.copyfile(f"{MYDIR}/{f}", f"{FIXTURE_FILES_DIR}{FIXTURE_FILES_PREFIX}-{f}")

    logging.info("Ensure local anvil server is not running")
    subprocess.run(["make", "kill-anvil"], cwd=f"{MYDIR}/../", check=True)

    logging.info("Starting local anvil server")
    subprocess.run(
        ["./run-local-anvil.sh", "-l", ANVIL_LOG_FILE, "-c", ANVIL_CFG_FILE],
        check=True,
        capture_output=True,
        cwd=f"{MYDIR}/../scripts",
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

    nodes = NODES.copy()

    logging.info("Create safe and modules for all the ids, store them in args files")
    nodes = create_local_safes(nodes, private_key)

    try:
        for node_id, node_args in nodes.items():
            (proc, api) = setup_node(node_args)
            nodes[node_id]["proc"] = proc
            nodes[node_id]["api"] = api

        logging.info(f"Wait for {len(nodes)} nodes to start up")

        for node_id, node_args in nodes.items():
            while True:
                with open(f"{node_args['dir']}.log", "r") as f:
                    logs = f.read()
                    if "still unfunded, " in logs or "node is funded" in logs:
                        logging.info(f"Node {node_id} up")
                        break

                sleep(0.1)

        logging.info("Funding nodes")
        funding_nodes(private_key)

        logging.info("Collect node information")
        for node_id, node_args in nodes.items():
            (peer_id, address) = collect_node_information(node_args)
            nodes[node_id]["peer_id"] = peer_id
            nodes[node_id]["address"] = address

        logging.info("Node setup finished, waiting 10 seconds before proceeding")
        sleep(10)

        yield nodes
    except Exception as e:
        logging.error(f"Creating a 7 node cluster - FAILED: {e}")
    finally:
        logging.debug("Tearing down the 7 node cluster")
        for node_id, node_args in nodes.items():
            cleanup_node(node_args)
        pass
