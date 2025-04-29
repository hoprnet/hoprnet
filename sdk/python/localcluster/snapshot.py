import shutil
from pathlib import Path

from .cluster import Cluster
from .constants import ANVIL_FOLDER, ANVIL_FOLDER_NAME, NODE_NAME_PREFIX, logging

EXPECTED_FILES_FOR_SNAPSHOT = [
    "db/hopr_index.db",
    "db/hopr_index.db-shm",
    "db/hopr_index.db-wal",
    "db/hopr_logs.db",
    "db/hopr_logs.db-shm",
    "db/hopr_logs.db-wal",
]


class Snapshot:
    def __init__(self, anvil_port: int, parent_dir: Path, cluster: Cluster):
        self.anvil_port = anvil_port
        self.parent_dir = parent_dir
        self.cluster = cluster

    def create(self):
        logging.info("Taking snapshot")

        # delete old snapshot
        shutil.rmtree(self.sdir, ignore_errors=True)

        # create new snapshot
        self.sdir.mkdir(parents=True, exist_ok=True)

        # copy anvil files
        shutil.copytree(ANVIL_FOLDER, self.sdir.joinpath(ANVIL_FOLDER_NAME))
        shutil.rmtree(ANVIL_FOLDER, ignore_errors=True)

        # copy node data and env files
        for i in range(self.cluster.size):
            source_dir: Path = self.parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            target_dir = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            db_target_dir = target_dir.joinpath("db/")

            db_target_dir.mkdir(parents=True, exist_ok=True)

            for file in EXPECTED_FILES_FOR_SNAPSHOT:
                shutil.copy(source_dir.joinpath(file), db_target_dir)

            for file in source_dir.glob("*.cfg.yaml"):
                shutil.copy(file, target_dir)

            shutil.copy(source_dir.joinpath("./hoprd.id"), target_dir)
            shutil.copy(source_dir.joinpath("./.env"), target_dir)

    def reuse(self):
        logging.info("Re-using snapshot")

        # remove all files and folder in self.dir which are not snapshot

        for entry in self.parent_dir.glob("*"):
            if entry.is_dir():
                if entry.name == "snapshot":
                    continue
                shutil.rmtree(entry, ignore_errors=True)
            else:
                entry.unlink(missing_ok=True)

        # copy snapshot files
        shutil.copytree(self.sdir, self.parent_dir, dirs_exist_ok=True)

    @property
    def usable(self):
        expected_files = [self.sdir.joinpath(ANVIL_FOLDER_NAME)]
        for i in range(self.cluster.size):
            node_dir = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            expected_files.extend([node_dir.joinpath(file) for file in EXPECTED_FILES_FOR_SNAPSHOT])

            if not any(node_dir.glob("*.cfg.yaml")):
                logging.warning(f"Cannot find *.cfg.yaml in {node_dir}")
                return False

        for f in expected_files:
            if not f.exists():
                logging.warning(f"Cannot find {f} in snapshot")
                return False

        return True

    @property
    def sdir(self):
        return self.parent_dir.joinpath("snapshot")
