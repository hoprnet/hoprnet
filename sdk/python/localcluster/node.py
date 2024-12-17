import asyncio
import logging
import os
from pathlib import Path
from subprocess import STDOUT, Popen, run

from ..api import HoprdAPI
from . import utils
from .constants import MAIN_DIR, NODE_NAME_PREFIX, PASSWORD, PORT_BASE, PWD


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
        cfg_file: str,
    ):
        # initialized
        self.id = id
        self.host_addr: str = host_addr
        self.api_token: str = api_token
        self.network: str = network

        # optional
        self.cfg_file: str = cfg_file

        # generated
        self.safe_address: str = None
        self.module_address: str = None
        self.proc: Popen = None

        # private
        self.peer_id: str = None
        self.address: str = None
        self.dir: Path = None
        self.cfg_file_path: Path = None
        self.api_port: int = 0
        self.p2p_port: int = 0
        self.anvil_port: int = 0

        self.prepare()

    @property
    def api(self):
        return HoprdAPI(f"http://{self.host_addr}:{self.api_port}", self.api_token)

    def prepare(self):
        self.anvil_port = PORT_BASE
        self.dir = MAIN_DIR.joinpath(f"{NODE_NAME_PREFIX}_{self.id}")
        self.cfg_file_path = MAIN_DIR.joinpath(self.cfg_file)
        self.api_port = PORT_BASE + (self.id * 10) + 1
        self.p2p_port = PORT_BASE + (self.id * 10) + 2

    def load_addresses(self):
        loaded_env = load_env_file(f"{self.dir}.env")
        self.safe_address = loaded_env.get("HOPRD_SAFE_ADDRESS")
        self.module_address = loaded_env.get("HOPRD_MODULE_ADDRESS")
        if self.safe_address is None or self.module_address is None:
            raise ValueError(
                "Critical addresses are missing in the environment file.")

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
                f"{self.dir}.id",
                "--contracts-root",
                "./ethereum/contracts",
                "--hopr-amount",
                "20000.0",
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
            cwd=PWD.parent,
        )

        for el in res.stdout.split("\n"):
            if el.startswith("safe 0x"):
                self.safe_address = el.split()[-1]

            if el.startswith("node_module 0x"):
                self.module_address = el.split()[-1]

        # store the addresses in a file which can be loaded later
        if self.safe_address is not None and self.module_address is not None:
            with open(f"{self.dir}.env", "w") as env_file:
                env_file.write(f"HOPRD_SAFE_ADDRESS={self.safe_address}\n")
                env_file.write(f"HOPRD_MODULE_ADDRESS={self.module_address}\n")
            return True

        logging.error(
            f"Failed to create safe for node {self.id}: {res.stdout} - {res.stderr}")
        return False

    def setup(self, password: str, config_file: Path, dir: Path):
        trace_telemetry = "true" if os.getenv(
            "TRACE_TELEMETRY") is not None else "false"
        log_level = "trace" if os.getenv(
            "TRACE_TELEMETRY") is not None else "debug"

        api_token_param = f"--api-token={self.api_token}" if self.api_token else "--disableApiAuthentication"
        custom_env = {
            "RUST_LOG": ",".join(
                [
                    log_level,
                    "libp2p_swarm=info",
                    "libp2p_mplex=info",
                    "multistream_select=info",
                    "isahc=error",
                    "sea_orm=warn",
                    "sqlx=warn",
                    "hyper_util=warn",
                    "libp2p_tcp=info",
                    "libp2p_dns=info",
                    "hickory_resolver=warn",
                ]
            ),
            "RUST_BACKTRACE": "full",
            "HOPRD_USE_OPENTELEMETRY": trace_telemetry,
            "OTEL_SERVICE_NAME": f"hoprd-{self.p2p_port}",
            "TOKIO_CONSOLE_BIND": f"localhost:{self.p2p_port+100}",
        }
        loaded_env = load_env_file(f"{self.dir}.env")

        cmd = [
            "hoprd",
            "--announce",
            "--api",
            "--init",
            "--testAnnounceLocalAddresses",
            "--testPreferLocalAddresses",
            f"--apiPort={self.api_port}",
            f"--data={self.dir}",
            f"--host={self.host_addr}:{self.p2p_port}",
            f"--identity={self.dir}.id",
            f"--network={self.network}",
            f"--password={password}",
            f"--protocolConfig={config_file}",
            f"--provider=http://127.0.0.1:{self.anvil_port}",
            api_token_param,
        ]
        if self.cfg_file_path is not None:
            cmd += [f"--configurationFilePath={self.cfg_file_path}"]

        with open(f"{self.dir}.log", "w") as log_file:
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
            peers = [p.peer_id for p in await asyncio.wait_for(self.api.peers(), timeout=20)]
            missing_peers = [p for p in required_peers if p not in peers]
            ready = len(missing_peers) == 0

            if not ready:
                await asyncio.sleep(0.2)

        return ready

    def clean_up(self):
        self.proc.kill()

    @classmethod
    def fromConfig(cls, index: int, config: dict, api_token: dict, network: dict):
        token = api_token["default"]
        network = network["default"]

        if "api_token" in config:
            token = config["api_token"]

        if "network" in config:
            network = config["network"]

        return cls(index, token, config["host"], network, config["config_file"])

    async def links(self):
        addresses = await self.api.addresses()
        print(f"\t{self}")
        print(f"\t\tPeer Id:\t{addresses.hopr}")
        print(f"\t\tAddress:\t{addresses.native}")
        print(
            f"\t\tRest API:\thttp://{self.host_addr}:{self.api_port}/scalar | http://{self.host_addr}:{self.api_port}/swagger-ui/index.html")
        print(
            f"\t\tAdmin UI:\thttp://{self.host_addr}:4677/?apiEndpoint=http://{self.host_addr}:{self.api_port}&apiToken={self.api_token}", end="\n\n")

    def __eq__(self, other):
        return self.peer_id == other.peer_id

    def __str__(self):
        return f"node@{self.host_addr}:{self.api_port}"
