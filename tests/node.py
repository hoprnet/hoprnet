from pathlib import Path
from subprocess import Popen

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

    def clean_up(self):
        self.proc.kill()

    def __str__(self):
        return f"node@{self.host_addr}:{self.p2p_port}"
