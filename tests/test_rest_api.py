import random

import aiohttp
import pytest

from sdk.python.localcluster.node import Node
from tests.conftest import nodes_with_auth


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
async def test_hoprd_rest_api_should_reject_connection_without_any_auth(swarm7: dict[str, Node], peer: str):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"

    async with aiohttp.ClientSession() as s:
        assert (await s.get(url).status) == 401


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
async def test_hoprd_rest_api_should_reject_connection_with_invalid_token(peer: str, swarm7: dict[str, Node]):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"
    invalid_token = swarm7[peer].api_token.swapcase()
    headers = {"X-Auth-Token": invalid_token}

    async with aiohttp.ClientSession(headers=headers) as s:
        assert (await s.get(url).status) == 401


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
async def test_hoprd_rest_api_should_accept_connection_with_valid_token(peer: str, swarm7: dict[str, Node]):
    url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v3/node/version"
    headers = {"X-Auth-Token": swarm7[peer].api_token}

    async with aiohttp.ClientSession(headers=headers) as s:
        assert (await s.get(url).status) == 200
