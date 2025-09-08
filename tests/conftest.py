import asyncio
import logging
import os
import re
from subprocess import PIPE, STDOUT, CalledProcessError, Popen

import pytest

from sdk.python import localcluster
from sdk.python.localcluster.constants import PWD
from sdk.python.localcluster.node import Node

from .find_port import find_available_port_block

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


@pytest.fixture(scope="session")
async def base_port(request):
    base_port_env = os.environ.get("HOPR_SMOKETEST_BASE_PORT")

    if base_port_env is None:
        base_port = find_available_port_block(3000, 4000, 30)
    else:
        base_port = int(base_port_env)

    if base_port is None:
        pytest.fail("No available base port found")  # ty: ignore[call-non-callable]
    logging.info(f"Using base port: {base_port}")
    yield base_port


@pytest.fixture(scope="session")
async def swarm7(request, base_port):
    params_path = PWD.joinpath("sdk/python/localcluster.test.params.yml")
    try:
        cluster_and_anvil = await localcluster.bringup(
            params_path, test_mode=True, fully_connected=False, use_nat=False, base_port=base_port
        )

        if cluster_and_anvil is None:
            pytest.fail("Failed to bring up the cluster")
            return

        cluster, anvil = cluster_and_anvil
        yield cluster.nodes

        cluster.clean_up()
        anvil.kill()
    except RuntimeError:
        pytest.fail("Failed to bring up the cluster")  # ty: ignore[call-non-callable]


@pytest.fixture(scope="function")
async def swarm7_reset(swarm7: dict[str, Node]):
    yield

    logging.debug("Resetting swarm7 nodes")
    try:
        await asyncio.gather(*[node.api.reset_tickets_statistics() for node in swarm7.values()])
    except Exception as e:
        logging.error(f"Error resetting tickets statistics in teardown: {e}")


def to_ws_url(host, port, args: list[tuple[str, str]]):
    return f"ws://{host}:{port}/api/v4/session/websocket?" + "&".join(f"{a[0]}={a[1]}" for a in args)


def run_hopli_cmd(cmd: list[str], custom_env):
    env = os.environ | custom_env
    proc = Popen(cmd, env=env, stdout=PIPE, stderr=STDOUT, bufsize=0, cwd=PWD)
    # filter out ansi color codes
    color_regex = re.compile(r"\x1b\[\d{,3}m")
    with proc.stdout:  # ty: ignore[invalid-context-manager]
        for line in iter(proc.stdout.readline, b""):  # ty: ignore[possibly-unbound-attribute]
            logging.debug("[Hopli] %r", color_regex.sub("", line.decode("utf-8")[:-1]))
    retcode = proc.wait()
    if retcode:
        raise CalledProcessError(retcode, cmd)
