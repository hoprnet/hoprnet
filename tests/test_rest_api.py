import random

import aiohttp
import pytest

from sdk.python.localcluster.node import Node
from tests.conftest import nodes_with_auth


@pytest.mark.usefixtures("swarm7_reset")
class TestRestApiWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
    async def test_hoprd_rest_api_should_reject_connection_without_any_auth(self, swarm7: dict[str, Node], peer: str):
        url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v4/node/version"

        async with aiohttp.ClientSession() as s:
            assert (await s.get(url)).status == 401

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
    async def test_hoprd_rest_api_should_reject_connection_with_invalid_token(self, peer: str, swarm7: dict[str, Node]):
        url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v4/node/version"
        invalid_token = swarm7[peer].api_token.swapcase()
        headers = {"X-Auth-Token": invalid_token}

        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 401

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
    async def test_hoprd_rest_api_should_accept_connection_with_valid_token(self, peer: str, swarm7: dict[str, Node]):
        url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v4/node/version"
        headers = {"X-Auth-Token": swarm7[peer].api_token}

        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 200

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
    async def test_metric_endpoint_accepts_plain_text_header(self, peer: str, swarm7: dict[str, Node]):
        url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/metrics"
        headers = {"X-Auth-Token": swarm7[peer].api_token}

        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 200

        headers["accept"] = "text/plain"
        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 200

        headers["accept"] = "application/json"
        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 406

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(nodes_with_auth(), 1))
    async def test_info_endpoint_accepts_application_json_header(self, peer: str, swarm7: dict[str, Node]):
        url = f"http://{swarm7[peer].host_addr}:{swarm7[peer].api_port}/api/v4/node/info"
        headers = {"X-Auth-Token": swarm7[peer].api_token}

        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 200

        headers["accept"] = "text/plain"
        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 406

        headers["accept"] = "application/json"
        async with aiohttp.ClientSession(headers=headers) as s:
            assert (await s.get(url)).status == 200
