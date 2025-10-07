import asyncio
import logging
import os
import tempfile
from pathlib import Path
from subprocess import STDOUT, Popen, run
from typing import Optional

from api_lib.headers.authorization import Bearer

from ..api import HoprdAPI
from . import utils
from .constants import (
    MAIN_DIR,
    NODE_NAME_PREFIX,
    OPEN_CHANNEL_FUNDING_VALUE_HOPR,
    PASSWORD,
    PWD,
)


def load_env_file(env_file: str) -> dict:
    env = {}
    try:
        with open(env_file, "r") as f:
            for line in f:
                if line.startswith("#"):
                    continue
                key, value = line.strip().split("=", 1)
                env[key] = value
    except FileNotFoundError:
        logging.error(f"Environment file {env_file} not found.")
    except ValueError:
        logging.error(f"Incorrect format in environment file {env_file}.")
    return env


class Node:
    def __init__(
        self,
        id: int,
        api_token: str,
        host_addr: str,
        network: str,
        identity_path: str,
        cfg_file: str,
        base_port: int,
        cluster_size: int,
        api_addr: Optional[str] = None,
        use_nat: bool = False,
        remove_temp_data: bool = True,
    ):
        # initialized
        self.id = id
        self.host_addr: str = host_addr
        self.api_token: str = api_token
        self.network: str = network
        self.identity_path: str = identity_path
        self.use_nat: bool = use_nat
        self.remove_temp_data: bool = remove_temp_data
        self.base_port: int = base_port
        self.temp_data_dir = tempfile.TemporaryDirectory(prefix=f"{NODE_NAME_PREFIX}_{self.id}_")

        # optional
        self.cfg_file: str = cfg_file
        self.api_addr: str = api_addr
        if api_addr is None:
            self.api_addr = host_addr

        # generated
        self.safe_address: str = None
        self.module_address: str = None
        self.proc: Popen = None

        # private
        self.address: str = None
        self.dir: Path = None
        self.cfg_file_path: Path = None
        self.api_port: int = 0
        self.p2p_port: int = 0
        self.anvil_port: int = 0
        self.tokio_console_port: int = 0

        self.prepare(cluster_size)

    @property
    def api(self):
        return HoprdAPI(f"http://{self.api_addr}:{self.api_port}", Bearer(self.api_token), "/api/v4")

    def prepare(self, cluster_size: int):
        self.dir = MAIN_DIR.joinpath(f"{NODE_NAME_PREFIX}_{self.id}")
        self.cfg_file_path = MAIN_DIR.joinpath(self.cfg_file)
        self.anvil_port = self.base_port
        self.api_port = self.base_port + self.id
        self.p2p_port = self.api_port + cluster_size
        self.tokio_console_port = self.p2p_port + cluster_size

        logging.info(
            f"Node {self.id} ports: "
            + f"api {self.api_port}, "
            + f"p2p {self.p2p_port}, "
            + f"tokio console {self.tokio_console_port}"
        )

    def load_native_address(self):
        logging.debug(f"Reading node_id {self}")

        safe_custom_env = {
            "ETHERSCAN_API_KEY": "anykey",
            "IDENTITY_PASSWORD": PASSWORD,
            "PATH": os.environ["PATH"],
        }

        res = run(
            [
                "hopli",
                "identity",
                "read",
                "--identity-from-path",
                self.dir.joinpath("hoprd.id"),
            ],
            env=os.environ | safe_custom_env,
            check=True,
            capture_output=True,
            text=True,
            cwd=PWD,
        )

        for el in res.stdout.split("\n"):
            if el.startswith("Identity addresses:"):
                logging.debug(f"Node {self.id} identity read line: {el}")
                self.address = el.split("[")[-1].rstrip("]")

    def load_addresses(self):
        loaded_env = load_env_file(self.dir.joinpath(".env"))
        self.safe_address = loaded_env.get("HOPRD_SAFE_ADDRESS")
        self.module_address = loaded_env.get("HOPRD_MODULE_ADDRESS")
        if self.safe_address is None or self.module_address is None:
            raise ValueError("Critical addresses are missing in the environment file.")

    def create_local_safe(self, anvil_config: Path):
        logging.debug(f"Creating safe and module for {self}")

        private_key = utils.load_private_key(anvil_config)

        safe_custom_env = {
            "ETHERSCAN_API_KEY": "anykey",
            "IDENTITY_PASSWORD": PASSWORD,
            "MANAGER_PRIVATE_KEY": private_key,
            "PRIVATE_KEY": private_key,
            "PATH": os.environ["PATH"],
        }

        res = run(
            [
                "hopli",
                "safe-module",
                "create",
                "--network",
                self.network,
                "--identity-from-path",
                self.dir.joinpath("hoprd.id"),
                "--contracts-root",
                "./ethereum/contracts",
                "--hopr-amount",
                "200000000000.0",
                "--native-amount",
                "10.0",
                "--private-key",
                "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
                "--manager-private-key",
                "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
                "--provider-url",
                f"http://127.0.0.1:{self.anvil_port}",
            ],
            env=os.environ | safe_custom_env,
            check=True,
            capture_output=True,
            text=True,
            cwd=PWD,
        )

        for el in res.stdout.split("\n"):
            if el.startswith("safe 0x"):
                self.safe_address = el.split()[-1]

            if el.startswith("node_module 0x"):
                self.module_address = el.split()[-1]

        # store the addresses in a file which can be loaded later
        if self.safe_address is not None and self.module_address is not None:
            self.dir.mkdir(parents=True, exist_ok=True)
            with open(self.dir.joinpath(".env"), "w") as env_file:
                env_file.write(f"HOPRD_SAFE_ADDRESS={self.safe_address}\n")
                env_file.write(f"HOPRD_MODULE_ADDRESS={self.module_address}\n")
            return True

        logging.error(f"Failed to create safe for node {self.id}: {res.stdout} - {res.stderr}")
        return False

    def setup(self, password: str, config_file: Path, dir: Path, log_tag: str):
        trace_telemetry = "true" if os.getenv("TRACE_TELEMETRY") is not None else "false"
        log_level = "trace" if os.getenv("TRACE_TELEMETRY") is not None else "info"

        api_token_param = f"--api-token={self.api_token}" if self.api_token else "--disableApiAuthentication"
        custom_env = {
            "RUST_LOG": ",".join(
                [
                    log_level,
                    "hyper_util=warn",
                    "hickory_resolver=warn",
                    "hopr_transport_session=debug",
                    "hopr_protocol_session=debug",
                    "hopr_network_types=debug",
                    "hopr_protocol_session=debug",
                    "isahc=error",
                    "sea_orm=warn",
                    "sqlx=warn",
                ]
            ),
            "RUST_BACKTRACE": "full",
            "HOPR_TEST_DISABLE_CHECKS": "true",
            "HOPRD_USE_OPENTELEMETRY": trace_telemetry,
            "OTEL_SERVICE_NAME": f"hoprd-{self.p2p_port}",
            "TOKIO_CONSOLE_BIND": f"localhost:{self.tokio_console_port}",
            "HOPRD_NAT": "true" if self.use_nat else "false",
            "HOPR_CAPTURE_PACKETS": self.dir.joinpath(f"capture_{self.id}.pcap"),
        }
        loaded_env = load_env_file(self.dir.joinpath(".env"))

        cmd = [
            "hoprd",
            "--announce",
            "--api",
            "--init",
            "--testAnnounceLocalAddresses",
            "--testPreferLocalAddresses",
            f"--apiHost={self.api_addr}",
            f"--apiPort={self.api_port}",
            f"--data={self.temp_data_dir.name}",
            f"--host={self.host_addr}:{self.p2p_port}",
            f"--identity={self.dir.joinpath('hoprd.id')}",
            f"--network={self.network}",
            f"--password={password}",
            f"--protocolConfig={config_file}",
            f"--provider=http://127.0.0.1:{self.anvil_port}",
            api_token_param,
        ]
        if self.cfg_file_path is not None:
            cmd += [f"--configurationFilePath={self.cfg_file_path}"]
        log_file_name = f"hoprd.{log_tag}.log" if log_tag else "hoprd.log"

        with open(self.dir.joinpath(log_file_name), "w") as log_file:
            self.proc = Popen(
                cmd,
                stdout=log_file,
                stderr=STDOUT,
                env=os.environ | custom_env | loaded_env,
                cwd=dir,
            )

        return self.proc is not None

    async def all_peers_connected(self, required_peers):
        ready = False

        while not ready:
            # we choose a long timeout here to accommodate the node just starting
            peers_info = await asyncio.wait_for(self.api.peers(), timeout=10)
            logging.debug(f"Peers info on {self.id}: {peers_info}")

            # filter out peers that are not well-connected yet
            connected_peers = [p.address for p in peers_info if p.quality >= 0.1]
            connected_peers.sort()
            logging.debug(f"Peers connected on {self.id}: {connected_peers}")

            missing_peers = [p for p in required_peers if p not in connected_peers]
            logging.debug(f"Peers not connected on {self.id}: {missing_peers}")

            ready = missing_peers == []

            if not ready:
                await asyncio.sleep(0.5)
            else:
                logging.info(f"All peers connected on {self.id}")

        return ready

    def clean_up(self):
        self.proc.kill()
        if self.remove_temp_data:
            try:
                self.temp_data_dir.cleanup()
            except OSError as e:
                logging.warning(f"Failed to cleanup temporary directory for node {self.id}: {e}")

    @classmethod
    def fromConfig(
        cls,
        index: int,
        config: dict,
        defaults: dict,
        network: str,
        use_nat: bool,
        exposed: bool,
        base_port: int,
        cluster_size: int,
    ):
        token = config.get("api_token", defaults.get("api_token"))

        return cls(
            index,
            token,
            config["host"],
            network,
            config["identity_path"],
            config["config_file"],
            cluster_size=cluster_size,
            api_addr="0.0.0.0" if exposed else None,
            use_nat=use_nat,
            base_port=base_port,
        )

    async def connect_peers(self, addresses: list[str]):
        tasks = []

        for address in addresses:
            if address == self.address:
                continue
            tasks.append(asyncio.create_task(self.api.open_channel(address, OPEN_CHANNEL_FUNDING_VALUE_HOPR)))

        await asyncio.gather(*tasks)

    async def links(self):
        addresses = await self.api.addresses()
        admin_ui_params = f"apiEndpoint=http://{self.api_addr}:{self.api_port}&apiToken={self.api_token}"

        output_strings = []

        output_strings.append(f"\t{self}")
        output_strings.append(f"\t\tAddress:\t{addresses.native}")
        output_strings.append(
            f"\t\tRest API:\thttp://{self.api_addr}:{self.api_port}/scalar | http://{self.api_addr}:{self.api_port}/swagger-ui/index.html"
        )
        output_strings.append(f"\t\tAdmin UI:\thttp://{self.host_addr}:4677/?{admin_ui_params}")
        output_strings.append(f"\t\tProcess ID:\t{self.proc.pid if self.proc else 'N/A'}\n\n")

        return "\n".join(output_strings)

    def __eq__(self, other):
        return self.address == other.address

    def __str__(self):
        return f"node @ {self.api_addr}:{self.api_port}"
