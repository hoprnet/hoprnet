import asyncio
import itertools
import logging
import os
import random
import re
from subprocess import PIPE, STDOUT, CalledProcessError, Popen

import pytest

from sdk.python import localcluster
from sdk.python.localcluster.constants import PWD
from sdk.python.localcluster.node import Node

# prepend the timestamp in front of any log line
logging.basicConfig(format="%(asctime)s %(message)s")


def pytest_addoption(parser: pytest.Parser):
    parser.addoption(
        "--stress-request-count",
        action="store",
        default=1000,
        type=int,
        help="Number of requests performed by each source node towards the stressed target",
    )
    parser.addoption(
        "--stress-sources",
        action="store",
        type=str,
        help="""The JSON string containing the list of dicts with 'url' and 'token' keys for each 
        stress test source node""",
    )
    parser.addoption(
        "--stress-target",
        action="store",
        type=str,
        help="""The JSON string containing the dict with 'url' and 'token' keys for the stressed
        target node""",
    )


# the test framework uses the following folder structure:
#
# /tmp/hopr-smoke-test/ - parent directory for all test related files
# /tmp/hopr-smoke-test/snapshot/ - directory for snapshot files which can be re-used
# /tmp/hopr-smoke-test/${SUITE_NAME}/ - directory for a specific test suite
# /tmp/hopr-smoke-test/${SUITE_NAME}/hopr-node_*.id - identity file for nodes
# /tmp/hopr-smoke-test/${SUITE_NAME}/hopr-node_*.log - log file for nodes
# /tmp/hopr-smoke-test/${SUITE_NAME}/anvil.cfg - anvil configuration file
# /tmp/hopr-smoke-test/${SUITE_NAME}/anvil.log - anvil configuration file


def barebone_nodes():
    """Nodes using only the barebones config without any strategies"""
    return ["1", "2", "3", "4"]


def nodes_with_auth():
    """All nodes within the same network as specified in the swarm7 fixture"""
    return ["1", "3", "4"]


def default_nodes():
    """A node that uses the default strategies"""
    return ["5"]


def nodes_with_lower_outgoing_win_prob():
    """Nodes with outgoing ticket winning probability"""
    return ["6"]


def random_distinct_tuples_from(values: list, count: int, tuple_length: int = 2):
    if tuple_length < 2:
        raise ValueError("Tuple length must be at least 2")
    return random.sample([tuple(combination) for combination in itertools.permutations(values, tuple_length)], count)


@pytest.fixture(scope="module")
async def swarm7(request):
    # path is related to where the test is run. Most likely the root of the repo
    cluster, anvil = await localcluster.bringup(
        "./sdk/python/localcluster.params.yml", test_mode=True, fully_connected=False
    )

    yield cluster.nodes

    cluster.clean_up()
    anvil.kill()


@pytest.fixture(scope="module", autouse=True)
async def teardown(swarm7: dict[str, Node]):
    yield

    try:
        await asyncio.gather(*[node.api.reset_tickets_statistics() for node in swarm7.values()])
    except Exception as e:
        logging.error(f"Error resetting tickets statistics in teardown: {e}")

    try:
        await asyncio.gather(*[node.api.messages_pop_all(None) for node in swarm7.values()])
    except Exception as e:
        logging.error(f"Error popping all messages in teardown: {e}")


def to_ws_url(host, port, args: list[tuple[str, str]]):
    return f"ws://{host}:{port}/api/v3/session/websocket?" + "&".join(f"{a[0]}={a[1]}" for a in args)


def run_hopli_cmd(cmd: list[str], custom_env):
    env = os.environ | custom_env
    proc = Popen(cmd, env=env, stdout=PIPE, stderr=STDOUT, bufsize=0, cwd=PWD)
    # filter out ansi color codes
    color_regex = re.compile(r"\x1b\[\d{,3}m")
    with proc.stdout:
        for line in iter(proc.stdout.readline, b""):
            logging.debug("[Hopli] %r", color_regex.sub("", line.decode("utf-8")[:-1]))
    retcode = proc.wait()
    if retcode:
        raise CalledProcessError(retcode, cmd)
