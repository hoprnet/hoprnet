import random
import json
import logging
import re
import os
from subprocess import run, Popen, PIPE, STDOUT, CalledProcessError

import pytest

from .conftest import (
    FIXTURES_PREFIX,
    INPUT_DEPLOYMENTS_SUMMARY_FILE,
    NETWORK1,
    NETWORK2,
    PASSWORD,
    PWD,
    anvil_log_file,
    anvil_cfg_file,
    barebone_nodes,
    fixtures_dir,
    load_private_key,
)
from .test_integration import (
    balance_str_to_int,
)
from .node import Node

FIXTURES_PREFIX_NEW = "hopr-node-new_"
PASSWORD_NEW = "e2e-test-new"

PORT_BASE = 19200
ANVIL_ENDPOINT = f"http://127.0.0.1:{PORT_BASE}"

def run_cast_cmd(cmd: str, params: list[str]):
    cast_cmd = ["cast", cmd, "-r", ANVIL_ENDPOINT] + params
    logging.info("Running cast command: %s", ' '.join(cast_cmd))
    try:
        result = run(cast_cmd, check=True, capture_output=True)
        return result
    except CalledProcessError as e:
        logging.error("Error executing cast command: %s", str(e))
        raise

def run_hopli_cmd(cmd: list[str], custom_env):
    env = os.environ | custom_env
    proc = Popen(cmd, env=env, stdout=PIPE, stderr=STDOUT, bufsize=0)
    # filter out ansi color codes
    color_regex = re.compile(r"\x1b\[\d{,3}m")
    with proc.stdout:
        for line in iter(proc.stdout.readline, b""):
            logging.info("[Hopli] %r", color_regex.sub("", line.decode("utf-8")[:-1]))
    retcode = proc.wait()
    if retcode:
        raise CalledProcessError(retcode, cmd)


def faucet(private_key: str, hopr_amount: str, native_amount: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    cmd = [
        "hopli",
        "faucet",
        "--network",
        NETWORK1,
        "--identity-prefix",
        FIXTURES_PREFIX,
        "--identity-directory",
        test_dir,
        "--contracts-root",
        "./ethereum/contracts",
        "--hopr-amount",
        hopr_amount,
        "--native-amount",
        native_amount,
        "--provider-url",
        ANVIL_ENDPOINT,
    ]
    run_hopli_cmd(cmd, custom_env)


def manager_deregsiter(private_key: str, node_addr: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "network-registry",
            "manager-deregister",
            "--network",
            NETWORK1,
            "--node-address",
            node_addr,
            "--contracts-root",
            "./ethereum/contracts",
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        custom_env,
    )


def manager_register(private_key: str, node_addr: str, safe_addr: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "network-registry",
            "manager-register",
            "--network",
            NETWORK1,
            "--contracts-root",
            "./ethereum/contracts",
            "--node-address",
            node_addr,
            "--safe-address",
            safe_addr,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        custom_env,
    )


def manager_force_sync(private_key: str, safes: str, eligibility: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "network-registry",
            "manager-force-sync",
            "--network",
            NETWORK1,
            "--safe-address",
            safes,
            "--contracts-root",
            "./ethereum/contracts",
            "--eligibility",
            eligibility,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        custom_env,
    )


def new_identity(extra_prefix: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "identity",
            "create",
            "--identity-prefix",
            FIXTURES_PREFIX_NEW + extra_prefix,
            "--identity-directory",
            test_dir,
            "--number",
            "1",
        ],
        custom_env,
    )


def read_identity(extra_prefix: str, pwd: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": pwd,
        "PATH": os.environ["PATH"],
    }
    res = run(
        [
            "hopli",
            "identity",
            "read",
            "--identity-prefix",
            FIXTURES_PREFIX_NEW + extra_prefix,
            "--identity-directory",
            test_dir,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

    for el in res.stdout.decode("utf-8").split("\n"):
        for p in el.split(": ["):
            if p.startswith("0x"):
                return p.split("]")[0]


def update_identity(extra_prefix: str, old_pwd: str, new_pwd: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": old_pwd,
        "NEW_IDENTITY_PASSWORD": new_pwd,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "identity",
            "update",
            "--identity-prefix",
            FIXTURES_PREFIX_NEW + extra_prefix,
            "--identity-directory",
            test_dir,
        ],
        custom_env,
    )


def create_safe_module(extra_prefix: str, private_key: str, manager_private_key: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "MANAGER_PRIVATE_KEY": manager_private_key,
        "PATH": os.environ["PATH"],
    }
    res = run(
        [
            "hopli",
            "safe-module",
            "create",
            "--network",
            NETWORK1,
            "--identity-prefix",
            FIXTURES_PREFIX_NEW + extra_prefix,
            "--identity-directory",
            test_dir,
            "--contracts-root",
            "./ethereum/contracts",
            "--allowance",
            "10.5",
            "--native-amount",
            "0.1",
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

    safe_address: str = None
    module_address: str = None

    for el in res.stdout.decode("utf-8").split("\n"):
        logging.info(el)
        if el.startswith("safe 0x"):
            safe_address = el.split()[-1]

        if el.startswith("node_module 0x"):
            module_address = el.split()[-1]
    return (safe_address, module_address)


def migrate_safe_module(private_key: str, manager_private_key: str, safe: str, module: str):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "MANAGER_PRIVATE_KEY": manager_private_key,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "safe-module",
            "migrate",
            "--network",
            NETWORK2,
            "--identity-prefix",
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            test_dir,
            "--contracts-root",
            "./ethereum/contracts",
            "--safe-address",
            safe,
            "--module-address",
            module,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        custom_env,
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
@pytest.mark.xfail(reason="race-conditions lead to incorrect balances on nodes")
async def test_hopli_should_be_able_to_fund_nodes(peer: str, swarm7: dict[str, Node]):
    test_suite_name = __name__.split('.')[-1]
    private_key = load_private_key(test_suite_name)

    balance_before = await swarm7[peer].api.balances()
    logging.debug(f"balance_before of {peer} / {swarm7[peer].address}: {balance_before}")

    # fund node with 1 HOPR token and 10 native token
    faucet(private_key, "1.0", "10.0")

    balance_after = await swarm7[peer].api.balances()
    logging.debug(f"balance_after of {peer} / {swarm7[peer].address}: {balance_after}")

    # Check if `hopli faucet` funds node to the desired amount
    # on the native token
    if balance_str_to_int(balance_before.native) > 10 * 1e18:
        assert balance_str_to_int(balance_after.native) == balance_str_to_int(balance_before.native)
    else:
        assert balance_str_to_int(balance_after.native) == int(10 * 1e18)

    # on the HOPR token
    if balance_str_to_int(balance_before.hopr) > 1 * 1e18:
        assert balance_str_to_int(balance_after.hopr) == balance_str_to_int(balance_before.native)
    else:
        assert balance_str_to_int(balance_after.hopr) == int(1 * 1e18)


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hopli_should_be_able_to_deregister_nodes_and_register_it(peer: str, swarm7: dict[str, Node]):
    test_suite_name = __name__.split('.')[-1]
    private_key = load_private_key(test_suite_name)

    with open(INPUT_DEPLOYMENTS_SUMMARY_FILE, "r") as file:
        address_data: dict = json.load(file)
        network_registry_contract = address_data["networks"][NETWORK1]["addresses"]["network_registry"]

    res_before = run_cast_cmd(
        "call", [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address]
    )

    # check the returned value is address safe
    assert res_before.stdout.decode("utf-8").split("\n")[0].lower() == swarm7[peer].safe_address.lower()

    # remove node from the network registry
    manager_deregsiter(private_key, swarm7[peer].address)

    # Check if nodes are removed from the network
    run_cast_cmd("code", [network_registry_contract])
    res_after_deregster = run_cast_cmd(
        "call", [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address]
    )

    # check the returned value is address zero
    assert (
        res_after_deregster.stdout.decode("utf-8").split("\n")[0].lower()
        == "0x0000000000000000000000000000000000000000"
    )

    # register node to the network registry
    manager_register(private_key, swarm7[peer].address, swarm7[peer].safe_address)

    # Check if nodes are removed from the network
    run_cast_cmd("code", [network_registry_contract])
    res_after_register = run_cast_cmd(
        "call", [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address]
    )

    # check the returned value is address safe
    assert res_after_register.stdout.decode("utf-8").split("\n")[0].lower() == swarm7[peer].safe_address.lower()


@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
async def test_hopli_should_be_able_to_sync_eligibility_for_all_nodes(peer: str, swarm7: dict[str, Node]):
    test_suite_name = __name__.split('.')[-1]
    private_key = load_private_key(test_suite_name)

    # remove all the nodes from the network registry
    manager_force_sync(private_key, swarm7[peer].safe_address, "true")


@pytest.mark.asyncio
async def test_hopli_create_update_read_identity():
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)
    extra_prefix = "one"

    # create a new identity
    new_identity(extra_prefix)

    # read the identity
    res_first_read = read_identity(extra_prefix, PASSWORD)

    # udpate identtiy password
    update_identity(extra_prefix, PASSWORD, PASSWORD_NEW)

    # still can read the identity
    res_second_read = read_identity(extra_prefix, PASSWORD_NEW)

    assert res_first_read == res_second_read

    # Remove the created identity
    run(["rm", "-f", test_dir.joinpath(f"{FIXTURES_PREFIX_NEW}{extra_prefix}0.id")], check=True, capture_output=True)


@pytest.mark.asyncio
async def test_hopli_should_be_able_to_create_safe_module(swarm7: dict[str, Node]):
    test_suite_name = __name__.split('.')[-1]
    test_dir = fixtures_dir(test_suite_name)
    manager_private_key = load_private_key(test_suite_name)
    private_key = load_private_key(test_suite_name, 1)
    extra_prefix = "two"

    # READ CONTRACT ADDRESS
    with open(INPUT_DEPLOYMENTS_SUMMARY_FILE, "r") as file:
        address_data: dict = json.load(file)
        network_registry_contract_1 = address_data["networks"][NETWORK1]["addresses"]["network_registry"]
        # network_registry_contract_2 = address_data["networks"][NETWORK2]["addresses"]["network_registry"]

    # create identity
    new_identity(extra_prefix)

    # read the identity
    new_node = read_identity(extra_prefix, PASSWORD)

    # create safe and module for
    new_safe_module = create_safe_module(extra_prefix, private_key, manager_private_key)
    node_balance = run_cast_cmd("balance", [new_node])
    safe_code = run_cast_cmd("code", [new_safe_module[0]])
    module_code = run_cast_cmd("code", [new_safe_module[1]])

    # Check the node node is registered with the new safe
    res_check_created_safe_registration = run_cast_cmd(
        "call", [network_registry_contract_1, "nodeRegisterdToAccount(address)(address)", new_node]
    )
    new_safe_module_address = new_safe_module[0].lower()
    res_registration = res_check_created_safe_registration.stdout.decode("utf-8").split("\n")[0].lower()
    assert res_registration == new_safe_module_address

    # Remove the created identity
    run(["rm", "-f", test_dir.joinpath(f"{FIXTURES_PREFIX_NEW}{extra_prefix}0.id")], check=True, capture_output=True)
