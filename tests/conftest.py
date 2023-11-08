import logging
import os
import random
import shutil
import subprocess

import pytest
from hopr import HoprdAPI

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
        default=200,
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


FIXTURE_FILES_DIR = "/tmp/"
FIXTURE_FILES_PREFIX = "hopr-smoke-test"

NODE_NAME_PREFIX = "hopr-smoke-test-node"


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
    },
    "6": {
        "api_port": 19096,
        "p2p_port": 13306,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_6",
        "host_addr": "127.0.0.1",
        "network": "anvil-localhost2"
    },
    "7": {
        "api_port": 19097,
        "p2p_port": 13307,
        "api_token": DEFAULT_API_TOKEN,
        "dir": f"{FIXTURE_FILES_DIR}{NODE_NAME_PREFIX}_7",
        "host_addr": "localhost",
    },
}


def cleanup_node(args):
    proc = args.proc
    proc.kill()
def setup_node(args):
    logging.info(f"Setting up a node with configuration: {args}")
    log_file = open(f"{args.dir}.log", 'w')
    network = args.network if args.network else "anvil-localhost"
    api_token_param = f"--api-token={args.api_token}" if args.api_token else "--disableApiAuthentication"
    env = {**os.environ,
           'DEBUG': "hopr*",
           'NODE_ENV'="development",
           'HOPRD_HEARTBEAT_INTERVAL'="2500",
           'HOPRD_HEARTBEAT_THRESHOLD'="2500",
           'HOPRD_HEARTBEAT_VARIANCE'="1000",
           'HOPRD_NETWORK_QUALITY_THRESHOLD'="0.3",
           'NODE_OPTIONS'="--experimental-wasm-modules"}
    cmd = [
    "node packages/hoprd/lib/main.cjs",
     f"--network={network}",
     f"--data="{args.dir}",
     f"--host="{args.host_addr}:{args.p2p_port}",
     "--identity="{args.dir}.id",
     "--init",
     "--password="{args.password}",
     "--api",
     "--apiPort="{args.api_port}"",
     "--testAnnounceLocalAddresses",
     "--disableTicketAutoRedeem",
     "--testPreferLocalAddresses",
     "--testUseWeakCrypto",
     "--announce",
    api_token_param
    ]
    if args.cfg_file:
        cmd + f"--configurationFilePath={args.cfg_file}"

    logging.info(f"Starting up a node with cmd: {cmd} and env {env}")
    proc = Popen(cmd, stdout=log_file, stderr=subprocess.STDOUT, env=env)
    return proc


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

@pytest.fixture(scope="module")
def swarm7(request):
    logging.info(f"Using the random seed: {SEED}")

    log_file_path = f"/tmp/hopr-smoke-{request.module.__name__}-setup.log"

    for f in ["node_5.cfg.yaml"]:
        shutil.copyfile(f"./tests/{f}", f"{FIXTURE_FILES_DIR}/{FIXTURE_FILES_PREFIX}-{f}")

    # TODO: start anvil
    # TODO: update protocol config
    # TODO: reuse identities
    # TODO: create local safes
    # TODO: start nodes
    try:
    nodes = NODES.copy()
    for node_id, node_args in NODES.items():
        proc = setup_node(node_args)
        nodes[node_id]["proc"] = proc
        nodes[node_id]["api"] = HoprdAPI(f"http://localhost:{node_args.api_port}", node_args.api_token)

    # TODO: wait for nodes
    # TODO: fund nodes
    # TODO: wait for port bindings
    # TODO: restart node 1 and wait

    yield nodes
    except Exception:
        logging.error("Creating a 7 node cluster - FAILED")
    finally:
        logging.debug("Tearing down the 7 node cluster")
        for node_id, node_args in nodes.items():
            cleanup_node(node_args)
        pass
