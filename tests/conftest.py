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
TICKET_PRICE_PER_HOP = 100 # modified later using API call


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


DEFAULT_API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"
NODES = {
    "1": {
        "p2p_port": 19091,
        "api_port": 13301,
        "peer_id": "12D3KooWKSzQgdszZzipRVGSRwBcC3etYwjSmqqTqcySn97EGWTm",
        "address": "0x7d1e530e9c82c21b75644a2c23402aa858ae4a69",
    },
    "2": {
        "p2p_port": 19092,
        "api_port": 13302,
        "peer_id": "12D3KooWLWoHJjaS1z9cXn19DE9gPrSbYHkf7CHMbUtLUqbZKDby",
        "address": "0x1b482420afa04aec1ef0e4a00c18451e84466c75",
    },
    "3": {
        "p2p_port": 19093,
        "api_port": 13303,
        "peer_id": "12D3KooWJ4E4q6wr8nzXyRKnAofQSeoGoRFRRtQJK3jCpLtLNVZj",
        "address": "0x05b17e37fd43c18741877fca80846ad8c84aa750",
    },
    "4": {
        "p2p_port": 19094,
        "api_port": 13304,
        "peer_id": "12D3KooWA494BRhXs2DpMm5e2DWkPZcot3WpYwB4KBj2udP9xvPC",
        "address": "0xcc70a22331998454160472f097acb43ca9b1e646",
    },
    "5": {
        "p2p_port": 19095,
        "api_port": 13305,
        "peer_id": "12D3KooWQCHVYhdnLT76rhHoFUu3b4L2aUiY8o1y8erhAwg8evFx",
        "address": "0xe4bb1970e6c9e5689c5ef68ee2545b4366c49be4",
    },
    "6": {
        "p2p_port": 19096,
        "api_port": 13306,
        "peer_id": "12D3KooWHc2LPyvYGLJbHoQeJUBXBjDRfMY7msobsPoWj8rCAGHr",
        "address": "0xf90c1eb2557a443c2b27d399afac075fa752cd92",
    },
    "7": {
        "p2p_port": 19097,
        "api_port": 13307,
        "peer_id": "12D3KooWGPUAcSaJhKBmmwP2Bz3PbZyrErosBmCbZKgPPt7XHwqh",
        "address": "0xe63ececd80c503548516e9e23ebb44d95c4d5ac2",
    },
}


def setup_node(*args, **kwargs):
    logging.info(f"Setting up a node with configuration: {args} and {kwargs}")
    pass


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

    try:
        logging.debug("Creating a 7 node cluster from bash")
        res = subprocess.run(
            f"./scripts/fixture_local_test_setup.sh --setup 2>&1 | tee {log_file_path}",
            shell=True,
            capture_output=True,
            check=True,
        )
        res.check_returncode()
        nodes = NODES.copy()

        for key in NODES.keys():
            port = NODES[key]["api_port"]
            nodes[key]["api"] = HoprdAPI(f"http://localhost:{port}", DEFAULT_API_TOKEN)

        yield nodes
    except Exception:
        logging.error("Creating a 7 node cluster from bash - FAILED")
    finally:
        logging.debug("Tearing down the 7 node cluster from bash")
        subprocess.run(
            f"./scripts/fixture_local_test_setup.sh --teardown 2>&1 | tee --append {log_file_path}",
            shell=True,
            capture_output=True,
            check=False,
        )
        pass
