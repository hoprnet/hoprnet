import json
import shutil
from enum import Enum, auto
from pathlib import Path
from subprocess import run

from .constants import PWD, logging


class AnvilState(Enum):
    DUMP = auto()
    LOAD = auto()


class Anvil:
    def __init__(self, log_file: Path, cfg_file: Path, state_file: Path, port: int, use_staking_proxy: bool):
        self.log_file = log_file
        self.cfg_file = cfg_file
        self.state_file = state_file
        self.port = port
        self.use_staking_proxy = use_staking_proxy

        self.log_file.parent.mkdir(parents=True, exist_ok=True)
        self.cfg_file.parent.mkdir(parents=True, exist_ok=True)
        self.state_file.parent.mkdir(parents=True, exist_ok=True)

    def run(self, state=AnvilState.DUMP):
        logging.info("Starting and waiting for local anvil server to be up " + f"({state.name.lower()} state enabled)")

        command = f"""
            bash scripts/run-local-anvil.sh
            {"-s " if state is AnvilState.LOAD else ""}
            -l {self.log_file}
            -c {self.cfg_file}
            -p {self.port}
            {"-ls" if state is AnvilState.LOAD else "-ds"} {self.state_file}
            {"-sp" if self.use_staking_proxy else ""}
            """

        run(
            command.split(),
            check=True,
            capture_output=True,
            cwd=PWD,
        )

    def mirror_contracts(self, src_file: Path, dest_file: Path, src_network: str, dest_network: str):
        logging.info("Mirror contract data because of anvil-deploy node only writing to localhost")
        shutil.copy(PWD.joinpath("scripts", "protocol-config-anvil.json"), dest_file)

        with open(src_file, "r") as file:
            src_data = json.load(file)

        with open(dest_file, "r") as file:
            dest_data = json.load(file)

        network_data = src_data["networks"][src_network]
        partial_network_data = {
            "environment_type": network_data["environment_type"],
            "indexer_start_block_number": 1,
            "addresses": network_data["addresses"],
        }
        new_network_data = dest_data["networks"][dest_network] | partial_network_data
        dest_data["networks"][dest_network] = new_network_data

        with open(dest_file, "w") as file:
            json.dump(dest_data, file, sort_keys=True)

    def kill(self):
        logging.info("Stopping all local anvil servers running")
        run(f"make -s kill-anvil port={self.port}".split(), cwd=PWD, check=False)
