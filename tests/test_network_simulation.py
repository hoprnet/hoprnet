import asyncio
import json
import logging
import os
import time
from contextlib import AsyncExitStack

import pytest
from websockets.asyncio.client import connect

from sdk.python.api.hopr import HoprdAPI
from sdk.python.localcluster.constants import TICKET_PRICE_PER_HOP
from sdk.python.localcluster.node import Node
from sdk.python import localcluster

from .conftest import to_ws_url
from .test_integration import create_channel

logging.basicConfig(format="%(asctime)s %(message)s")


PORT_BASE = 19300


@pytest.fixture(scope="module")
async def swarm7_docker(request):
    logging.info("Starting docker compose cluster")
    cluster, anvil = await localcluster.bringup(
        "./sdk/python/composecluster.params.yml", test_mode=True, fully_connected=True, docker_compose=True
    )

    yield cluster.nodes

    cluster.clean_up()


async def test_1hop_session(swarm7_docker: dict[str, Node]):
    assert False
