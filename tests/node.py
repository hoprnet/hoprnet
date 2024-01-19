import os
from pathlib import Path
from subprocess import STDOUT, Popen, run

from .hopr import HoprdAPI


class Node:
    def __init__(
        self,
        api_port: int,
        p2p_prt: int,
        api_token: str,
        dir: Path,
        host_addr: str,
        network: str,
        cfg_file: Path = None,
    ):
        # initialized
        self.api_port: int = api_port
        self.p2p_port: int = p2p_prt
        self.dir: Path = dir
        self.host_addr: str = host_addr
        self.api_token: str = api_token
        self.network: str = network

        # optional
        self.cfg_file: Path = cfg_file

        # generated
        self.safe_address: str = None
        self.module_address: str = None
        self.proc: Popen = None

        # private
        self.peer_id: str = None
        self.address: str = None

    @property
    def api(self):
        return HoprdAPI(f"http://{self.host_addr}:{self.api_port}", self.api_token)


    def create_local_safe(self, custom_env: dict):
        res = run(
            [
                "hopli",
                "create-safe-module",
                "--network",
                self.network,
                "--identity-from-path",
                f"{self.dir}.id",
                "--contracts-root",
                "./ethereum/contracts",
                "--hopr-amount",
                "20000.0",
            ],
            env=os.environ | custom_env,
            check=True,
            capture_output=True,
            text=True,
        )

        for el in res.stdout.split("\n"):
            if el.startswith("safe: address 0x"):
                self.safe_address = el.split()[-1]

            if el.startswith("module: address 0x"):
                self.module_address = el.split()[-1]

        return (self.address is not None and self.module_address is not None)


    def setup_node(self, password: str, config_file: Path, dir: Path):
        api_token_param = f"--api-token={self.api_token}" if self.api_token else "--disableApiAuthentication"
        custom_env = {
            "HOPRD_HEARTBEAT_INTERVAL": "2500",
            "HOPRD_HEARTBEAT_THRESHOLD": "2500",
            "HOPRD_HEARTBEAT_VARIANCE": "1000",
            "HOPRD_NETWORK_QUALITY_THRESHOLD": "0.3",
        }
        cmd = [
            "target/debug/hoprd",
            "--announce",
            "--api",
            "--disableTicketAutoRedeem",
            "--init",
            "--testAnnounceLocalAddresses",
            "--testPreferLocalAddresses",
            "--testUseWeakCrypto",
            f"--apiPort={self.api_port}",
            f"--data={self.dir}",
            f"--host={self.host_addr}:{self.p2p_port}",
            f"--identity={self.dir}.id",
            f"--moduleAddress={self.module_address}",
            f"--network={self.network}",
            f"--password={password}",
            f"--safeAddress={self.safe_address}",
            f"--protocolConfig={config_file}",
            api_token_param,
        ]
        if self.cfg_file is not None:
            cmd += [f"--configurationFilePath={self.cfg_file}"]

        with open(f"{self.dir}.log", "w") as log_file:
            self.proc = Popen(
                cmd,
                stdout=log_file,
                stderr=STDOUT,
                env=os.environ | custom_env,
                cwd=dir,
            )

        return (self.proc is not None)

    def clean_up(self):
        self.proc.kill()

    def __str__(self):
        return f"node@{self.host_addr}:{self.p2p_port}"
