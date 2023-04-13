import logging
import subprocess

import pytest

def pytest_addoption(parser):
    parser.addoption(
        '--stress-seq-request-count', action='store', default=200, help='Number of sequential requests in the stress test'
    )
    parser.addoption(
        '--stress-par-request-count', action='store', default=200, help='Number of parallel requests in the stress test'
    )


@pytest.fixture
def cmd_line_args(request):
    args = {}
    args['stress_seq_request_count'] = request.config.getoption('--stress-seq-request-count')
    args['stress_par_request_count'] = request.config.getoption('--stress-par-request-count')
    return args


DEFAULT_API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"
NODES = {
    "1": {
        "p2p_port": 19091,
        "api_port": 13301,
        "private_key": "0x1f5b172a64947589be6e279fbcbc09aca6e623a64a92aa359fae9c6613b7e801",
    },
    "2": {
        "p2p_port": 19092,
        "api_port": 13302,
        "private_key": "0xcb9c3533beb75b996b6c77150ecda32134d13710a16121f04dc591113329cd7c",
    },
    "3": {
        "p2p_port": 19093,
        "api_port": 13303,
        "private_key": "0x9a96a7711e2e9c9f71767bb9f248f699b29aebe7f590de8eeec0e71796b869e0",
    },
    "4": {
        "p2p_port": 19094,
        "api_port": 13304,
        "private_key": "0x7dea49b4dbeea4dcbbb9d071bc7212347748dc3a2f16896f504417236b6adb84",
    },
    "5": {
        "p2p_port": 19095,
        "api_port": 13305,
        "private_key": "0x800fee12d472c1a8448b786eb9e5d6c7f643c78b9727032893da9a6a55db288b",
    },
    "6": {
        "p2p_port": 19096,
        "api_port": 13306,
        "private_key": "0x79b94be0c06dac87139c54416228dcacfb084c6884bbf4e48fff4cab8f40baa6",
    },
    "7": {
        "p2p_port": 19097,
        "api_port": 13307,
        "private_key": "0x9b813edd8a85cffbe3cd2e242dc0992cfa04be15caa9f50b0b03b5ebcb2f770a",
    },
}


def setup_node(*args, **kwargs):
    logging.info(f"Setting up a node with configuration: {args} and {kwargs}")
    pass


def test_sanity():
    assert len(NODES.keys()) == len(set([i["private_key"] for i in NODES.values()])), "All private keys must be unique"

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
def setup_7_nodes():
    try:
        logging.info("Creating a 7 node cluster from source")
        subprocess.run(
            "./scripts/fixture_local_test_setup.sh --skip-cleanup",
            shell=True,
            capture_output=True,
            check=True,
        )
        yield NODES
    finally:
        logging.info("Tearing down the 7 node cluster from source")
        subprocess.run(
            "./scripts/fixture_local_test_setup.sh --just-cleanup",
            shell=True,
            capture_output=True,
            check=True,
        )
