import asyncio
import os
import shutil
from pathlib import Path
from subprocess import run

from . import utils
from .constants import (
    IDENTITY_PREFIX,
    MAIN_DIR,
    NETWORK,
    NODE_NAME_PREFIX,
    PASSWORD,
    PORT_BASE,
    PWD,
    logging,
)
from .node import Node

GLOBAL_TIMEOUT = 60


class Cluster:
    def __init__(self, config: dict, anvil_config: Path, protocol_config: Path):
        self.anvil_config = anvil_config
        self.protocol_config = protocol_config
        self.nodes: dict[str, Node] = {}
        index = 1

        for network_name, params in config["networks"].items():
            for alias, node in params["nodes"].items():
                self.nodes[str(index)] = Node.fromConfig(index, alias, node, config["api_token"], network_name)
                index += 1

    def clean_up(self):
        logging.info(f"Tearing down the {self.size} nodes cluster")
        [node.clean_up() for node in self.nodes.values()]

    def create_safes(self):
        logging.info("Creating safe and modules for all the ids, store them in args files")
        for node in self.nodes.values():
            assert node.create_local_safe(self.anvil_config)

    async def shared_bringup(self, skip_funding: bool = False):
        logging.info("Setting up nodes with protocol config files")
        for node in self.nodes.values():
            logging.debug(f"Setting up {node}")
            node.setup(PASSWORD, self.protocol_config, PWD)

        # WAIT FOR NODES TO BE UP
        logging.info(f"Waiting up to {GLOBAL_TIMEOUT}s for nodes to start up")
        nodes_readyness = await asyncio.gather(*[node.api.startedz(GLOBAL_TIMEOUT) for node in self.nodes.values()])
        for node, res in zip(self.nodes.values(), nodes_readyness):
            if res:
                logging.debug(f"Node {node} up")
            else:
                logging.error(f"Node {node} not started after {GLOBAL_TIMEOUT} seconds")

        if not all(nodes_readyness):
            logging.critical("Not all nodes are started, interrupting setup")
            raise RuntimeError

        if not skip_funding:
            self.fund_nodes()
            return

        # WAIT FOR NODES TO BE UP
        logging.info(f"Waiting up to {GLOBAL_TIMEOUT}s for nodes to be ready")
        nodes_readyness = await asyncio.gather(*[node.api.readyz(GLOBAL_TIMEOUT) for node in self.nodes.values()])
        for node, res in zip(self.nodes.values(), nodes_readyness):
            if res:
                logging.debug(f"Node {node} up")
            else:
                logging.error(f"Node {node} not ready after {GLOBAL_TIMEOUT} seconds")

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
        logging.info(f"Waiting up to {2 * GLOBAL_TIMEOUT}s for nodes to connect to all peers")

        tasks = []
        for node in self.nodes.values():
            required_peers = [n.peer_id for n in self.nodes.values() if n != node and n.network == node.network]
            tasks.append(asyncio.create_task(node.all_peers_connected(required_peers)))

        await asyncio.wait_for(asyncio.gather(*tasks), 2 * GLOBAL_TIMEOUT)

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
                NETWORK,
                "--identity-prefix",
                IDENTITY_PREFIX,
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
            cwd=PWD,
        )

    def copy_identities(self):
        logging.info("Using pre-generated identities and configs")

        # prepare folders
        for node_id in range(self.size):
            MAIN_DIR.joinpath(f"{NODE_NAME_PREFIX}_{node_id+1}").mkdir(parents=True, exist_ok=True)

        # Remove old identities
        for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}/*.id"):
            os.remove(f)
        logging.info(f"Removed '*.id' files in {MAIN_DIR} subfolders")

        # Remove old logs
        for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}/*.log"):
            os.remove(f)
        logging.info(f"Removed '*.log' files in {MAIN_DIR} subfolders")

        # Copy new identity files
        for idx, node in enumerate(self.nodes.values(), start=1):
            shutil.copy(
                PWD.joinpath(node.identity_path),
                MAIN_DIR.joinpath(f"{NODE_NAME_PREFIX}_{idx}", "hoprd.id"),
            )
        logging.info(f"Copied '*.id' files to {MAIN_DIR}")

        # Copy new config files
        for f in PWD.joinpath("sdk").glob("*.cfg.yaml"):
            shutil.copy(f, MAIN_DIR.joinpath(f.name))
        logging.info(f"Copied '*.cfg.yaml' files to {MAIN_DIR}")

    def load_addresses(self):
        for node in self.nodes.values():
            node.load_addresses()

    def get_safe_and_module_addresses(self):
        for node in self.node.values():
            node.get_safe_and_module_addresses()

    async def alias_peers(self):
        logging.info("Aliasing every other node")
        aliases_dict = {node.peer_id: node.alias for node in self.nodes.values()}

        for node in self.nodes.values():
            await node.alias_peers(aliases_dict)

    async def connect_peers(self):
        logging.info("Creating a channel to every other node")
        peer_ids = [node.peer_id for node in self.nodes.values()]

        tasks = [node.connect_peers(peer_ids) for node in self.nodes.values()]
        await asyncio.gather(*tasks)

    async def links(self):
        print("")
        for node in self.nodes.values():
            await node.links()

    @property
    def size(self):
        return len(self.nodes)
