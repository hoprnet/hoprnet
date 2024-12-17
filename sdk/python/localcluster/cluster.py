import asyncio
import os
import shutil
from pathlib import Path
from subprocess import run

from . import utils
from .constants import (
    FIXTURES_PREFIX,
    MAIN_DIR,
    NETWORK1,
    NODE_NAME_PREFIX,
    PASSWORD,
    PORT_BASE,
    PREGENERATED_IDENTITIES_DIR,
    PWD,
    logging,
)
from .node import Node

API_TIMEOUT = 60


class Cluster:
    def __init__(self, config: dict, anvil_config: Path, protocol_config: Path):
        self.anvil_config = anvil_config
        self.protocol_config = protocol_config
        self.nodes = {str(idx): Node.fromConfig(
            idx, n, config["api_token"], config["network"]) for idx, n in enumerate(config["nodes"], start=1)}

    def clean_up(self):
        logging.info(f"Tearing down the {self.size} nodes cluster")
        [node.clean_up() for node in self.nodes.values()]

    def create_safes(self):
        logging.info(
            "Creating safe and modules for all the ids, store them in args files")
        for node in self.nodes.values():
            assert node.create_local_safe(self.anvil_config)

    async def shared_bringup(self, skip_funding: bool = False):
        logging.info("Setting up nodes with protocol config files")
        for node in self.nodes.values():
            logging.debug(f"Setting up {node}")
            node.setup(PASSWORD, self.protocol_config, PWD.parent)

        # WAIT FOR NODES TO BE UP
        logging.info(f"Waiting up to {API_TIMEOUT}s for nodes to start up")
        nodes_readyness = await asyncio.gather(*[node.api.startedz(API_TIMEOUT) for node in self.nodes.values()])
        for node, res in zip(self.nodes.values(), nodes_readyness):
            if res:
                logging.debug(f"Node {node} up")
            else:
                logging.error(
                    f"Node {node} not started after {API_TIMEOUT} seconds")

        if not all(nodes_readyness):
            logging.critical("Not all nodes are started, interrupting setup")
            raise RuntimeError

        if not skip_funding:
            # FUND NODES
            self.fund_nodes()

        # WAIT FOR NODES TO BE UP
        logging.info(f"Waiting up to {API_TIMEOUT}s for nodes to be ready")
        nodes_readyness = await asyncio.gather(*[node.api.readyz(API_TIMEOUT) for node in self.nodes.values()])
        for node, res in zip(self.nodes.values(), nodes_readyness):
            if res:
                logging.debug(f"Node {node} up")
            else:
                logging.error(
                    f"Node {node} not ready after {API_TIMEOUT} seconds")

        if not all(nodes_readyness):
            logging.critical("Not all nodes are ready, interrupting setup")
            raise RuntimeError

        for node in self.nodes.values():
            if addresses := await node.api.addresses():
                node.peer_id = addresses.hopr
                node.address = addresses.native
            else:
                logging.error(f"Node {node} did not return addresses")

        # WAIT FOR NODES TO CONNECT TO ALL PEERS
        logging.info(
            f"Waiting up to {API_TIMEOUT}s for nodes to connect to all peers")

        tasks = []
        for node in self.nodes.values():
            required_peers = [n.peer_id for n in self.nodes.values(
            ) if n != node and n.network == node.network]
            tasks.append(asyncio.create_task(
                node.all_peers_connected(required_peers)))
            
        nodes_connectivity = await asyncio.gather(*tasks)
        for node, res in zip(self.nodes.values(), nodes_connectivity):
            if res:
                logging.debug(f"Node {node} connected to all peers")
            else:
                logging.error(f"Node {node} did not connect to all peers")

        if not all(nodes_connectivity):
            logging.critical(
                "Not all nodes are connected to all peers, interrupting setup")
            raise RuntimeError

    def fund_nodes(self):
        logging.info("Funding nodes")

        private_key = utils.load_private_key(self.anvil_config)

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
                MAIN_DIR,
                "--contracts-root",
                "./ethereum/contracts",
                "--hopr-amount",
                "0.0",
                "--native-amount",
                "10.0",
                "--provider-url",
                f"http://127.0.0.1:{PORT_BASE}",
            ],
            env=os.environ | custom_env,
            check=True,
            capture_output=True,
            cwd=PWD.parent
        )

    def copy_identities(self):
        logging.info("Using pre-generated identities and configs")

        # Remove old identities
        for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}*.id"):
            os.remove(f)
        logging.info(f"Removed '*.id' files in {MAIN_DIR}")

        # Remove old logs
        for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}_*.log"):
            os.remove(f)
        logging.info(f"Removed '*.log' files in {MAIN_DIR}")

        # Copy new identity files
        for node_id in range(self.size):
            f = f"{NODE_NAME_PREFIX}_{node_id+1}.id"
            shutil.copy(
                PREGENERATED_IDENTITIES_DIR.joinpath(f),
                MAIN_DIR.joinpath(f),
            )
        logging.info(f"Copied '*.id' files to {MAIN_DIR}")

        # Copy new config files
        for f in PWD.glob("*.cfg.yaml"):
            shutil.copy(f, MAIN_DIR.joinpath(f.name))
        logging.info(f"Copied '*.cfg.yaml' files to {MAIN_DIR}")

    def load_addresses(self):
        for node in self.nodes.values():
            node.load_addresses()

    async def links(self):
        print('')
        for node in self.nodes.values():
            await node.links()

    @property
    def size(self):
        return len(self.nodes)
