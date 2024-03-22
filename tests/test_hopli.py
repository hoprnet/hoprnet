import asyncio
import random
import json
import re
import logging
from contextlib import AsyncExitStack, asynccontextmanager
import os
from subprocess import run

import pytest
import requests

from .conftest import (
    PASSWORD,
    NETWORK1,
    NETWORK2,
    ANVIL_CFG_FILE,
    ANVIL_LOG_FILE,
    FIXTURES_PREFIX,
    FIXTURES_DIR,
    ANVIL_ENDPOINT,
    DEPLOYMENTS_SUMMARY_FILE,
    PWD,
    API_TOKEN,
    OPEN_CHANNEL_FUNDING_VALUE_HOPR,
    TICKET_AGGREGATION_THRESHOLD,
    TICKET_PRICE_PER_HOP,
    default_nodes,
    fund_nodes,
    default_nodes_with_auth,
    passive_node,
    random_distinct_pairs_from,
)
from .test_integration import (
    balance_str_to_int,
)
from .hopr import HoprdAPI
from .node import Node

FIXTURES_PREFIX_NEW = "hopr-smoke-test-new"
PASSWORD_NEW = "e2e-test-new"

def faucet(private_key: str, hopr_amount: str, native_amount: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "faucet",
            "--network",
            NETWORK1,
            "--identity-prefix",
            FIXTURES_PREFIX,
            "--identity-directory",
            FIXTURES_DIR,
            "--contracts-root",
            "./ethereum/contracts",
            "--hopr-amount",
            hopr_amount,
            "--native-amount",
            native_amount,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def manager_deregsiter(private_key: str, node_addr: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run(
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
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def manager_register(private_key: str, node_addr: str, safe_addr: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run(
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
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def manager_force_sync(private_key: str, nodes:str, eligibility: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "MANAGER_PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "network-registry",
            "manager-force-sync",
            "--network",
            NETWORK1,
            "--safe-address",
            nodes,
            "--contracts-root",
            "./ethereum/contracts",
            "--eligibility",
            eligibility,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def new_identity():
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "identity",
            "create",
            "--identity-prefix",
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            FIXTURES_DIR,
            "--number",
            "1",
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def read_identity(pwd: str):
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
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            FIXTURES_DIR,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

    for el in res.stdout.decode('utf-8').split("\n"):
        for p in el.split(": ["):
            if p.startswith("0x"):
                return p.split("]")[0]

def update_identity(old_pwd: str, new_pwd: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": old_pwd,
        "NEW_IDENTITY_PASSWORD": new_pwd,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "identity",
            "update",
            "--identity-prefix",
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            FIXTURES_DIR,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

def create_safe_module(private_key: str, manager_private_key: str):
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
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            FIXTURES_DIR,
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

    for el in res.stdout.decode('utf-8').split("\n"):
        logging.info(el)
        if el.startswith("safe 0x"):
            safe_address = el.split()[-1]

        if el.startswith("node_module 0x"):
            module_address = el.split()[-1]
    return (safe_address, module_address)

def migrate_safe_module(private_key: str, manager_private_key: str, safe: str, module: str):
    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "IDENTITY_PASSWORD": PASSWORD,
        "PRIVATE_KEY": private_key,
        "MANAGER_PRIVATE_KEY": manager_private_key,
        "PATH": os.environ["PATH"],
    }
    run(
        [
            "hopli",
            "safe-module",
            "migrate",
            "--network",
            NETWORK2,
            "--identity-prefix",
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            FIXTURES_DIR,
            "--contracts-root",
            "./ethereum/contracts",
            "--safe-address",
            safe,
            "--module-address",
            module,
            "--provider-url",
            ANVIL_ENDPOINT,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
async def test_hopli_should_be_able_to_fund_nodes(peer: str, swarm7: dict[str, Node]):
    # READ AUTO_GENERATED PRIVATE-KEY FROM ANVIL CONFIGURATION
    with open(ANVIL_CFG_FILE, "r") as file:
        data: dict = json.load(file)
        private_key = data.get("private_keys", [""])[0]

    balance_before = await swarm7[peer].api.balances()
    logging.debug(f"balance_before of {peer}: {balance_before}")

    # fund node with 1 HOPR token and 10 native token
    faucet(private_key, "1.0", "10.0")

    balance_after = await swarm7[peer].api.balances()
    logging.debug(f"balance_after of {peer}: {balance_after}")

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
@pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
async def test_hopli_should_be_able_to_deregister_nodes_and_register_it(peer: str, swarm7: dict[str, Node]):
    # READ AUTO_GENERATED PRIVATE-KEY FROM ANVIL CONFIGURATION
    with open(ANVIL_CFG_FILE, "r") as file:
        data: dict = json.load(file)
        private_key = data.get("private_keys", [""])[0]
            
    with open(DEPLOYMENTS_SUMMARY_FILE, "r") as file:
        address_data: dict = json.load(file)
        network_registry_contract = address_data["networks"][NETWORK1]["addresses"]["network_registry"]

    res_before = run(["cast", "call", network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address], check=True, capture_output=True)

    # check the returned value is address safe
    assert res_before.stdout.decode('utf-8').split("\n")[0].lower() == swarm7[peer].safe_address.lower()

    # remove node from the network registry
    manager_deregsiter(private_key, swarm7[peer].address)

    # Check if nodes are removed from the network
    run(["cast", "code", network_registry_contract], check=True, capture_output=True)
    res_after_deregster = run(["cast", "call", network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address], check=True, capture_output=True)

    # check the returned value is address zero
    assert res_after_deregster.stdout.decode('utf-8').split("\n")[0].lower() == "0x0000000000000000000000000000000000000000"

    # register node to the network registry
    manager_register(private_key, swarm7[peer].address, swarm7[peer].safe_address)

    # Check if nodes are removed from the network
    run(["cast", "code", network_registry_contract], check=True, capture_output=True)
    res_after_regsiter = run(["cast", "call", network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address], check=True, capture_output=True)

    # check the returned value is address safe
    assert res_after_regsiter.stdout.decode('utf-8').split("\n")[0].lower() == swarm7[peer].safe_address.lower()

@pytest.mark.asyncio
@pytest.mark.parametrize("peer", random.sample(default_nodes(), 1))
async def test_hopli_should_be_able_to_sync_eligibility_for_all_nodes(peer: str, swarm7: dict[str, Node]):
    # READ AUTO_GENERATED PRIVATE-KEY FROM ANVIL CONFIGURATION
    with open(ANVIL_CFG_FILE, "r") as file:
        data: dict = json.load(file)
        private_key = data.get("private_keys", [""])[0]

    # remove all the nodes from the network registry
    manager_force_sync(private_key, swarm7[peer].safe_address, "true")

@pytest.mark.asyncio
async def test_hopli_create_update_read_identity():
    # create a new identity
    new_identity()

    # read the identity
    res_first_read = read_identity(PASSWORD)

    # udpate identtiy password
    update_identity(PASSWORD, PASSWORD_NEW)

    # still can read the identity
    res_second_read = read_identity(PASSWORD_NEW)

    assert res_first_read == res_second_read

    # Remove the created identity
    run(["rm", "-f", FIXTURES_DIR.joinpath(f"{FIXTURES_PREFIX_NEW}0.id")], check=True, capture_output=True)

@pytest.mark.asyncio
async def test_hopli_should_be_able_to_create_safe_module():
    # STOP OLD LOCAL ANVIL SERVER
    logging.info("Ensure local anvil server is not running")
    run(["make", "kill-anvil"], cwd=PWD.parent, check=True)

    # START NEW LOCAL ANVIL SERVER
    logging.info("Starting and waiting for local anvil server to be up")
    run(
        f"./run-local-anvil.sh -l {ANVIL_LOG_FILE} -c {ANVIL_CFG_FILE}".split(),
        check=True,
        capture_output=True,
        cwd=PWD.parent.joinpath("scripts"),
    )
    # DEPLOY A DIFFERENT LOCAL NETWORK
    run(f"make anvil-deploy-contracts network={NETWORK2} environment-type=local".split(), cwd=PWD.parent.joinpath("ethereum/contracts"), check=True)

    # READ AUTO_GENERATED PRIVATE-KEY FROM ANVIL CONFIGURATION
    with open(ANVIL_CFG_FILE, "r") as file:
        data: dict = json.load(file)
        manager_private_key = data.get("private_keys", [""])[0]
        private_key = data.get("private_keys", [""])[1]

    # READ CONTRACT ADDRESS
    with open(DEPLOYMENTS_SUMMARY_FILE, "r") as file:
        address_data: dict = json.load(file)
        network_registry_contract_1 = address_data["networks"][NETWORK1]["addresses"]["network_registry"]
        # network_registry_contract_2 = address_data["networks"][NETWORK2]["addresses"]["network_registry"]

    # create identity
    new_identity()

    # read the identity
    new_node = read_identity(PASSWORD)

    # create safe and module for 
    new_safe_module = create_safe_module(private_key, manager_private_key)
    node_balance = run(["cast", "balance", new_node], check=True, capture_output=True)
    logging .info(node_balance)
    safe_code = run(["cast", "code", new_safe_module[0]], check=True, capture_output=True)
    logging .info(safe_code)
    module_code = run(["cast", "code", new_safe_module[1]], check=True, capture_output=True)
    logging .info(module_code)

    # Check the node node is registered with the new safe
    res_check_created_safe_registration = run(["cast", "call", network_registry_contract_1, "nodeRegisterdToAccount(address)(address)", new_node], check=True, capture_output=True)
    assert res_check_created_safe_registration.stdout.decode('utf-8').split("\n")[0].lower() == new_safe_module[0].lower()

    # Remove the created identity
    run(["rm", "-f", FIXTURES_DIR.joinpath(f"{FIXTURES_PREFIX_NEW}0.id")], check=True, capture_output=True)