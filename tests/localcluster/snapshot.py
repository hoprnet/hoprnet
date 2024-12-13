import shutil
from pathlib import Path

from .anvil import Anvil
from .cluster import Cluster
from .constants import NODE_NAME_PREFIX, logging


class Snapshot:
    def __init__(self, anvil_port: int, parent_dir: Path, cluster: Cluster):
        self.anvil_port = anvil_port
        self.parent_dir = parent_dir
        self.cluster = cluster

    def create(self, anvil_file: Path):
        logging.info("Taking snapshot")

        # delete old snapshot
        shutil.rmtree(self.sdir, ignore_errors=True)

        # create new snapshot
        self.sdir.mkdir(parents=True, exist_ok=True)

        # stop anvil and nodes
        self.cluster.clean_up()
        Anvil.kill()

        # copy anvil state
        shutil.copy(anvil_file, self.sdir)

        # copy configuration files
        for f in self.parent_dir.glob("*.cfg.yaml"):
            shutil.copy(f, self.sdir)

        # copy protocol config file
        shutil.copy(self.parent_dir.joinpath(
            "protocol-config.json"), self.sdir)

        # copy node data and env files
        for i in range(self.cluster.size):
            node_dir = self.parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            node_target_dir = self.sdir.joinpath(
                f"{NODE_NAME_PREFIX}_{i+1}/db/")
            node_target_dir.mkdir(parents=True, exist_ok=True)

            shutil.copy(f"{node_dir}/db/hopr_index.db", node_target_dir)
            shutil.copy(f"{node_dir}/db/hopr_index.db-shm", node_target_dir)
            shutil.copy(f"{node_dir}/db/hopr_index.db-wal", node_target_dir)
            shutil.copy(f"{node_dir}.env", self.sdir)

    def reuse(self):
        logging.info("Re-using snapshot")

        # copy anvil state
        self.parent_dir.joinpath("anvil.state.json").unlink(missing_ok=True)
        shutil.copy(self.sdir.joinpath("anvil.state.json"), self.parent_dir)

        # copy configuration files
        for f in self.sdir.glob("*.cfg.yaml"):
            self.parent_dir.joinpath(f.name).unlink(missing_ok=True)
            shutil.copy(f, self.parent_dir)

        # copy protocol-config.json
        shutil.copy(self.sdir.joinpath(
            "protocol-config.json"), self.parent_dir)

        # copy node data
        for i in range(self.cluster.size):
            node_target_dir = self.parent_dir.joinpath(
                f"{NODE_NAME_PREFIX}_{i+1}/db/")
            node_dir: Path = self.sdir.joinpath(
                f"{NODE_NAME_PREFIX}_{i+1}/db/")

            shutil.rmtree(node_target_dir, ignore_errors=True)
            node_target_dir.mkdir(parents=True, exist_ok=False)

            shutil.copy(node_dir.joinpath(
                "hopr_index.db"), node_target_dir)
            shutil.copy(node_dir.joinpath(
                "hopr_index.db-shm"), node_target_dir)
            shutil.copy(node_dir.joinpath(
                "hopr_index.db-wal"), node_target_dir)

            self.parent_dir.joinpath(
                f"{NODE_NAME_PREFIX}_{i+1}.env").unlink(missing_ok=True)
            shutil.copy(self.sdir.joinpath(
                f"{NODE_NAME_PREFIX}_{i+1}.env"), self.parent_dir)

    @property
    def usable(self):
        expected_files = [
            "anvil.state.json",
            "barebone.cfg.yaml",
            "default.cfg.yaml",
            "protocol-config.json",
        ]
        for i in range(self.cluster.size):
            node_dir = f"{NODE_NAME_PREFIX}_{i+1}"
            expected_files.append(f"{node_dir}/db/hopr_index.db")
            expected_files.append(f"{node_dir}/db/hopr_index.db-shm")
            expected_files.append(f"{node_dir}/db/hopr_index.db-wal")
            expected_files.append(f"{node_dir}.env")

        for f in expected_files:
            file_path = self.sdir.joinpath(f)
            if not file_path.exists():
                logging.info(f"Cannot find {file_path} in snapshot")
                return False

        return False  # True

    @property
    def sdir(self):
        return self.parent_dir.joinpath("snapshot")
