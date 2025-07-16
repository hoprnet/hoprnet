import json
import logging
import os
import random
from pathlib import Path
from subprocess import CalledProcessError, run
from typing import Optional

import pytest

from sdk.python.localcluster.constants import (
    ANVIL_CONFIG_FILE,
    CONTRACTS_ADDRESSES,
    CONTRACTS_DIR,
    MAIN_DIR,
    NETWORK,
    PASSWORD,
)
from sdk.python.localcluster.node import Node
from sdk.python.localcluster.utils import load_private_key

from .conftest import barebone_nodes, run_hopli_cmd

FIXTURES_PREFIX_NEW = "hopr-node-new_"
PASSWORD_NEW = "e2e-test-new"


def generate_anvil_endpoint(base_port: int) -> str:
    return f"http://127.0.0.1:{base_port}"


def remove_identity(folder: Path, filename: str):
    run(["rm", "-f", folder.joinpath(filename)], check=True, capture_output=True)


def run_cast_cmd(cmd: str, params: list[str], base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

    cast_cmd = ["cast", cmd, "-r", anvil_endpoint] + params
    logging.info("Running cast command: %s", " ".join(cast_cmd))
    try:
        result = run(cast_cmd, check=True, capture_output=True)
        return result
    except CalledProcessError as e:
        logging.error("Error executing cast command: %s", str(e))
        raise


def faucet(private_key: str, hopr_amount: str, native_amount: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

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
        NETWORK,
        "--contracts-root",
        CONTRACTS_DIR,
        "--hopr-amount",
        hopr_amount,
        "--native-amount",
        native_amount,
        "--provider-url",
        anvil_endpoint,
    ]
    run_hopli_cmd(cmd, custom_env)


def manager_deregister(private_key: str, node_addr: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

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
            NETWORK,
            "--node-address",
            node_addr,
            "--contracts-root",
            CONTRACTS_DIR,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


def manager_register(private_key: str, node_addr: str, safe_addr: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

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
            NETWORK,
            "--contracts-root",
            CONTRACTS_DIR,
            "--node-address",
            node_addr,
            "--safe-address",
            safe_addr,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


def manager_force_sync(private_key: str, safes: str, eligibility: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

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
            NETWORK,
            "--safe-address",
            safes,
            "--contracts-root",
            CONTRACTS_DIR,
            "--eligibility",
            eligibility,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


def new_identity(extra_prefix: str):
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
            MAIN_DIR.joinpath("test_hopli"),
            "--number",
            "1",
        ],
        custom_env,
    )


def read_identity(extra_prefix: str, pwd: str):
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
            MAIN_DIR.joinpath("test_hopli"),
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
            MAIN_DIR.joinpath("test_hopli"),
        ],
        custom_env,
    )


def create_safe_module(extra_prefix: str, private_key: str, manager_private_key: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

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
            NETWORK,
            "--identity-prefix",
            FIXTURES_PREFIX_NEW + extra_prefix,
            "--identity-directory",
            MAIN_DIR.joinpath("test_hopli"),
            "--contracts-root",
            CONTRACTS_DIR,
            "--allowance",
            "10.5",
            "--native-amount",
            "0.1",
            "--provider-url",
            anvil_endpoint,
        ],
        env=os.environ | custom_env,
        check=True,
        capture_output=True,
    )

    safe_address: Optional[str] = None
    module_address: Optional[str] = None

    for el in res.stdout.decode("utf-8").split("\n"):
        logging.info(el)
        if el.startswith("safe 0x"):
            safe_address = el.split()[-1]

        if el.startswith("node_module 0x"):
            module_address = el.split()[-1]
    return (safe_address, module_address)


def migrate_safe_module(private_key: str, manager_private_key: str, safe: str, module: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

    new_network = "anvil-localhost2"

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
            new_network,
            "--identity-prefix",
            FIXTURES_PREFIX_NEW,
            "--identity-directory",
            MAIN_DIR.joinpath("test_hopli"),
            "--contracts-root",
            CONTRACTS_DIR,
            "--safe-address",
            safe,
            "--module-address",
            module,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


def manager_set_win_prob(private_key: str, win_prob: str, base_port: int):
    anvil_endpoint = generate_anvil_endpoint(base_port)

    custom_env = {
        "ETHERSCAN_API_KEY": "anykey",
        "PRIVATE_KEY": private_key,
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "win-prob",
            "set",
            "--network",
            NETWORK,
            "--winning-probability",
            win_prob,
            "--contracts-root",
            CONTRACTS_DIR,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


def get_win_prob(base_port):
    anvil_endpoint = generate_anvil_endpoint(base_port)

    custom_env = {
        "PATH": os.environ["PATH"],
    }
    run_hopli_cmd(
        [
            "hopli",
            "win-prob",
            "get",
            "--network",
            NETWORK,
            "--contracts-root",
            CONTRACTS_DIR,
            "--provider-url",
            anvil_endpoint,
        ],
        custom_env,
    )


@pytest.mark.usefixtures("swarm7_reset")
class TestHopliWithSwarm:
    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    @pytest.mark.xfail(reason="race-conditions lead to incorrect balances on nodes")
    async def test_hopli_should_be_able_to_fund_nodes(self, peer: str, swarm7: dict[str, Node], base_port: int):
        private_key = load_private_key(ANVIL_CONFIG_FILE)

        balance_before = await swarm7[peer].api.balances()
        logging.debug(f"balance_before of {peer} / {swarm7[peer].address}: {balance_before}")

        # fund node with 1 HOPR token and 10 native token
        faucet(private_key, "1.0", "10.0", base_port)

        balance_after = await swarm7[peer].api.balances()
        logging.debug(f"balance_after of {peer} / {swarm7[peer].address}: {balance_after}")

        # Check if `hopli faucet` funds node to the desired amount
        # on the native token
        if balance_before.native > 10:
            assert balance_after.native == balance_before.native
        else:
            assert balance_after.native == 10

        # on the HOPR token
        if balance_before.hopr > 1:
            assert balance_after.hopr == balance_before.hopr
        else:
            assert balance_after.hopr == 1

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hopli_should_be_able_to_deregister_nodes_and_register_it(
        self, peer: str, swarm7: dict[str, Node], base_port: int
    ):
        private_key = load_private_key(ANVIL_CONFIG_FILE)

        with open(CONTRACTS_ADDRESSES, "r") as file:
            address_data: dict = json.load(file)
            network_registry_contract = address_data["networks"][NETWORK]["addresses"]["network_registry"]

        res_before = run_cast_cmd(
            "call",
            [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address],
            base_port,
        )

        # check the returned value is address safe
        assert res_before.stdout.decode("utf-8").split("\n")[0].lower() == swarm7[peer].safe_address.lower()

        # remove node from the network registry
        manager_deregister(private_key, swarm7[peer].address, base_port)

        # Check if nodes are removed from the network
        run_cast_cmd("code", [network_registry_contract], base_port)
        res_after_deregster = run_cast_cmd(
            "call",
            [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address],
            base_port,
        )

        # check the returned value is address zero
        assert (
            res_after_deregster.stdout.decode("utf-8").split("\n")[0].lower()
            == "0x0000000000000000000000000000000000000000"
        )

        # register node to the network registry
        manager_register(private_key, swarm7[peer].address, swarm7[peer].safe_address, base_port)

        # Check if nodes are removed from the network
        run_cast_cmd("code", [network_registry_contract], base_port)
        res_after_register = run_cast_cmd(
            "call",
            [network_registry_contract, "nodeRegisterdToAccount(address)(address)", swarm7[peer].address],
            base_port,
        )

        # check the returned value is address safe
        assert res_after_register.stdout.decode("utf-8").split("\n")[0].lower() == swarm7[peer].safe_address.lower()

    @pytest.mark.asyncio
    @pytest.mark.parametrize("peer", random.sample(barebone_nodes(), 1))
    async def test_hopli_should_be_able_to_sync_eligibility_for_all_nodes(
        self, peer: str, swarm7: dict[str, Node], base_port: int
    ):
        private_key = load_private_key(ANVIL_CONFIG_FILE)

        # remove all the nodes from the network registry
        manager_force_sync(private_key, swarm7[peer].safe_address, "true", base_port)

    @pytest.mark.asyncio
    async def test_hopli_should_be_able_to_create_safe_module(self, swarm7: dict[str, Node], base_port: int):
        manager_private_key = load_private_key(ANVIL_CONFIG_FILE)
        private_key = load_private_key(ANVIL_CONFIG_FILE, 1)
        extra_prefix = "two"

        # READ CONTRACT ADDRESS
        with open(CONTRACTS_ADDRESSES, "r") as file:
            address_data: dict = json.load(file)
            network_registry_contract_1 = address_data["networks"][NETWORK]["addresses"]["network_registry"]
            # network_registry_contract_2 = address_data["networks"][NETWORK2]["addresses"]["network_registry"]

        # create identity
        new_identity(extra_prefix)

        # read the identity
        new_node = read_identity(extra_prefix, PASSWORD)

        # create safe and module
        safe_address, module_address = create_safe_module(extra_prefix, private_key, manager_private_key, base_port)
        run_cast_cmd("balance", [new_node], base_port)
        run_cast_cmd("code", [safe_address], base_port)
        run_cast_cmd("code", [module_address], base_port)

        # Check the node node is registered with the new safe
        res_check_created_safe_registration = run_cast_cmd(
            "call", [network_registry_contract_1, "nodeRegisterdToAccount(address)(address)", new_node], base_port
        )
        res_registration = res_check_created_safe_registration.stdout.decode("utf-8").split("\n")[0].lower()
        assert res_registration == safe_address.lower()

        # Remove the created identity
        remove_identity(MAIN_DIR.joinpath("test_hopli"), f"{FIXTURES_PREFIX_NEW}{extra_prefix}0.id")

    @pytest.mark.asyncio
    async def test_hopli_should_be_able_to_set_and_read_win_prob(self, swarm7: dict[str, Node], base_port: int):
        # READ CONTRACT ADDRESS
        with open(CONTRACTS_ADDRESSES, "r") as file:
            address_data: dict = json.load(file)
            win_prob_oracle = address_data["networks"][NETWORK]["addresses"]["winning_probability_oracle"]

        # get current win prob
        get_win_prob(base_port)
        old_win_prob = run_cast_cmd("call", [win_prob_oracle, "currentWinProb()()"], base_port)
        logging.info("old_win_prob %s", old_win_prob.stdout.decode("utf-8").split("is")[0].split("\n")[0].lower())
        assert (
            old_win_prob.stdout.decode("utf-8").split("is")[0].split("\n")[0].lower()
            == "0x00000000000000000000000000000000000000000000000000ffffffffffffff"
        )

        # set new win prob
        private_key = load_private_key(ANVIL_CONFIG_FILE)
        manager_set_win_prob(private_key, "0.5", base_port)

        # get new win prob
        get_win_prob(base_port)
        new_win_prob = run_cast_cmd("call", [win_prob_oracle, "currentWinProb()()"], base_port)
        logging.info("new_win_prob %s", new_win_prob.stdout.decode("utf-8").split("is")[0].split("\n")[0].lower())
        assert (
            new_win_prob.stdout.decode("utf-8").split("is")[0].split("\n")[0].lower()
            == "0x000000000000000000000000000000000000000000000000007fffffffffffff"
        )


@pytest.mark.asyncio
async def test_hopli_create_update_read_identity():
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
    remove_identity(MAIN_DIR.joinpath("test_hopli"), f"{FIXTURES_PREFIX_NEW}{extra_prefix}0.id")
