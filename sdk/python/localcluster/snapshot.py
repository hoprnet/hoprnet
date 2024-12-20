import shutil
from pathlib import Path

from .cluster import Cluster
from .constants import NODE_NAME_PREFIX, logging

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

    def create(self, anvil_file: Path):
        logging.info("Taking snapshot")

        # delete old snapshot
        shutil.rmtree(self.sdir, ignore_errors=True)

        # create new snapshot
        self.sdir.mkdir(parents=True, exist_ok=True)

        # copy anvil state
        shutil.copy(anvil_file, self.sdir)

        # copy configuration files
        for f in self.parent_dir.glob("*.cfg.yaml"):
            shutil.copy(f, self.sdir)

        # copy protocol config file
        shutil.copy(self.parent_dir.joinpath("protocol-config.json"), self.sdir)

        # copy node data and env files
        for i in range(self.cluster.size):
            source_dir: Path = self.parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            target_dir = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            db_target_dir = target_dir.joinpath("db/")

            db_target_dir.mkdir(parents=True, exist_ok=True)

            for file in EXPECTED_FILES_FOR_SNAPSHOT:
                shutil.copy(source_dir.joinpath(file), db_target_dir)

            shutil.copy(source_dir.joinpath("./hoprd.id"), target_dir)
            shutil.copy(source_dir.joinpath("./.env"), target_dir)

    def reuse(self):
        logging.info("Re-using snapshot")

        # copy anvil state
        shutil.copy(self.sdir.joinpath("anvil.state.json"), self.parent_dir)

        # copy configuration files
        for f in self.sdir.glob("*.cfg.yaml"):
            self.parent_dir.joinpath(f.name).unlink(missing_ok=True)
            shutil.copy(f, self.parent_dir)

        # copy protocol-config.json
        shutil.copy(self.sdir.joinpath("protocol-config.json"), self.parent_dir)

        # copy node data
        for i in range(self.cluster.size):
            source_dir: Path = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            target_dir = self.parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            db_target_dir = target_dir.joinpath("db/")

            shutil.rmtree(db_target_dir, ignore_errors=True)
            db_target_dir.mkdir(parents=True, exist_ok=True)

            for file in EXPECTED_FILES_FOR_SNAPSHOT:
                shutil.copy(source_dir.joinpath(file), db_target_dir)

            shutil.copy(source_dir.joinpath("./hoprd.id"), target_dir)
            shutil.copy(source_dir.joinpath("./.env"), target_dir)

    @property
    def usable(self):
        expected_files = [
            self.sdir.joinpath("anvil.state.json"),
            self.sdir.joinpath("barebone.cfg.yaml"),
            self.sdir.joinpath("default.cfg.yaml"),
            self.sdir.joinpath("protocol-config.json"),
        ]
        for i in range(self.cluster.size):
            node_dir = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i+1}")
            expected_files.extend([node_dir.joinpath(file) for file in EXPECTED_FILES_FOR_SNAPSHOT])

        for f in expected_files:
            if not f.exists():
                logging.info(f"Cannot find {f} in snapshot")
                return False

        return True

    @property
    def sdir(self):
        return self.parent_dir.joinpath("snapshot")
