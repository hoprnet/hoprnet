import logging
import subprocess

import pytest

DEFAULT_API_TOKEN = 'e2e-API-token^^'
PASSWORD = "e2e-test"
NODES = {
    "1": {
        "p2p_port": 19091,
        "api_port": 13301,
        "private_key": "0x1f5b172a64947589be6e279fbcbc09aca6e623a64a92aa359fae9c6613b7e801",
        "peer_id": "16Uiu2HAm2SxWfXGqFsem2cZVwPh56GgPrdaWsFk1ZPkLVZ5EWA3X",
    },
    "2": {
        "p2p_port": 19092,
        "api_port": 13302,
        "private_key": "0xcb9c3533beb75b996b6c77150ecda32134d13710a16121f04dc591113329cd7c",
        "peer_id": "16Uiu2HAkzBPGEw2sxS6dQrHXZ8TfQvpk7Tc2AfKs4uKGjQ2JDMrm",
    },
    "3": {
        "p2p_port": 19093,
        "api_port": 13303,
        "private_key": "0x9a96a7711e2e9c9f71767bb9f248f699b29aebe7f590de8eeec0e71796b869e0",
        "peer_id": "16Uiu2HAkzEnkW3xGJbvpXSXmvVR177LcR4Sw7z5S1ijuBcnbVFsV",
    },
    "4": {
        "p2p_port": 19094,
        "api_port": 13304,
        "private_key": "0x7dea49b4dbeea4dcbbb9d071bc7212347748dc3a2f16896f504417236b6adb84",
        "peer_id": "16Uiu2HAm3nRSB4rDdrvniV1X3myJ13H1xLZdVHDBbxFFfTtMthbD",
    },
    "5": {
        "p2p_port": 19095,
        "api_port": 13305,
        "private_key": "0x800fee12d472c1a8448b786eb9e5d6c7f643c78b9727032893da9a6a55db288b",
        "peer_id": "16Uiu2HAmVVcQUEHr1JzsBmyZkupWGxiwBtav6o5rfzKhKRUFVAD8",
    },
    "6": {
        "p2p_port": 19096,
        "api_port": 13306,
        "private_key": "0x79b94be0c06dac87139c54416228dcacfb084c6884bbf4e48fff4cab8f40baa6",
        "peer_id": "16Uiu2HAkxzXPsLwA5L7KaLK3NKrkkRqBYnZBP3Wv29A7q8m8QqQG",
    },
    "7": {
        "p2p_port": 19097,
        "api_port": 13307,
        "private_key": "0x9b813edd8a85cffbe3cd2e242dc0992cfa04be15caa9f50b0b03b5ebcb2f770a",
        "peer_id": "16Uiu2HAmHQJEHh9RD4fME6tyEDgyFZhNW2M4zR7sbpMkjJ6jGDbj",
    },
}


def setup_node(*args, **kwargs):
    logging.info(f"Setting up a node with configuration: {args} and {kwargs}")
    pass


def test_sanity():
    assert len(NODES.keys()) == len(set([i['private_key'] for i in NODES.values()])), "All private keys must be unique"

    assert len(NODES.keys()) == len(set([i['api_port'] for i in NODES.values()])), "All API ports must be unique"

    assert len(NODES.keys()) == len(set([i['p2p_port'] for i in NODES.values()])), "All p2p ports must be unique"


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
    log_file_path = f"/tmp/hopr-smoke-{request.module.__name__}.log"
    try:
        logging.info("Creating a 7 node cluster from source")
        subprocess.run(
            f"./scripts/fixture_local_test_setup.sh --skip-cleanup | tee {log_file_path}",
            shell=True,
            capture_output=True,
            check=True,
        )
        yield NODES
    finally:
        logging.info("Tearing down the 7 node cluster from source")
        subprocess.run('./scripts/fixture_local_test_setup.sh --just-cleanup',
                       shell=True,
                       capture_output=True,
                       check=True)
