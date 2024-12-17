import random

import pytest
import requests

from sdk.python.localcluster.node import Node
from tests.conftest import nodes_with_auth

# used by nodes to get unique port assignments
PORT_BASE = 19000


@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
def test_hoprd_rest_api_should_reject_connection_without_any_auth(swarm7: dict[str, Node], peer: str):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"

    r = requests.get(url)

    assert r.status_code == 401


@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
def test_hoprd_rest_api_should_reject_connection_with_invalid_token(peer: str, swarm7: dict[str, Node]):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"
    headers = {"X-Auth-Token": swarm7[peer].api_token.swapcase()}

    r = requests.get(url, headers=headers)

    assert r.status_code == 401


@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
def test_hoprd_rest_api_should_accept_connection_with_valid_token(peer: str, swarm7: dict[str, Node]):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"
    headers = {"X-Auth-Token": swarm7[peer].api_token}

    r = requests.get(url, headers=headers)

    assert r.status_code == 200
