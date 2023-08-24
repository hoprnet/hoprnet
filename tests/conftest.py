import logging
import subprocess

import pytest

LOCALHOST = "127.0.0.1"


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


DEFAULT_API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"
NODES = {
    "1": {
        "p2p_port": 19091,
        "api_port": 13301,
        "peer_id": "16Uiu2HAmUYnGY3USo8iy13SBFW7m5BMQvC4NETu1fGTdoB86piw7",
    },
    "2": {
        "p2p_port": 19092,
        "api_port": 13302,
        "peer_id": "16Uiu2HAmEH1GFK9TdMVusuBdLkEAdqzcYZqegWH9iBRThW6TmuUn",
    },
    "3": {
        "p2p_port": 19093,
        "api_port": 13303,
        "peer_id": "16Uiu2HAkyMzkTpuMtgGPQCuSnLWJGaWF4pbmCxBQ58Md9TqxSis2",
    },
    "4": {
        "p2p_port": 19094,
        "api_port": 13304,
        "peer_id": "16Uiu2HAmBDXcqtEp5RjTq6PKiJdfgYPNTkDqjmxBVAmvqoi5PLxk",
    },
    "5": {
        "p2p_port": 19095,
        "api_port": 13305,
        "peer_id": "16Uiu2HAm7exqjZrqS79AnrxmBBuioRbCdXu8BrwpCMtLKqAnAV1m",
    },
    "6": {
        "p2p_port": 19096,
        "api_port": 13306,
        "peer_id": "16Uiu2HAmUDhBPjFuDXUy1S65uBGAxGmUooH72433jTAkLvk1Cc4Y",
    },
    "7": {
        "p2p_port": 19097,
        "api_port": 13307,
        "peer_id": "16Uiu2HAmQWRq84LRiA41t4np7HkCTWKsLqF9xg8ct6dNBJzXvALU",
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
def setup_7_nodes(request):
    log_file_path = f"/tmp/hopr-smoke-{request.module.__name__}-setup.log"
    try:
        logging.info("Creating a 7 node cluster from source")
        res = subprocess.run(
            f"./scripts/fixture_local_test_setup.sh --skip-cleanup 2>&1 | tee {log_file_path}",
            shell=True,
            capture_output=True,
            check=True,
        )
        res.check_returncode()
        yield NODES
    except Exception:
        logging.info("Creating a 7 node cluster from source - FAILED")
    finally:
        logging.info("Tearing down the 7 node cluster from source")
        subprocess.run(
            "./scripts/fixture_local_test_setup.sh --just-cleanup",
            shell=True,
            capture_output=True,
            check=True,
        )
