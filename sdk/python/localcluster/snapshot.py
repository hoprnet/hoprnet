import shutil
from pathlib import Path

from .cluster import Cluster
from .constants import ANVIL_FOLDER, ANVIL_FOLDER_NAME, NODE_NAME_PREFIX, logging


class Snapshot:
    def __init__(self, anvil_port: int, parent_dir: Path, cluster: Cluster):
        self.anvil_port = anvil_port
        self.parent_dir = parent_dir
        self.cluster = cluster

    def create(self):
        logging.info(f"Taking snapshot (anvil port: {self.anvil_port}")

        # delete old snapshot
        shutil.rmtree(self.sdir, ignore_errors=True)

        # create new snapshot
        self.sdir.mkdir(parents=True, exist_ok=True)

        # copy configuration files
        for f in self.parent_dir.glob("*.cfg.yaml"):
            shutil.copy(f, self.sdir)

        # copy anvil files
        shutil.copytree(ANVIL_FOLDER, self.sdir.joinpath(ANVIL_FOLDER_NAME))
        shutil.rmtree(ANVIL_FOLDER, ignore_errors=True)

        # copy node data and env files
        for i in range(self.cluster.size):
            source_dir: Path = self.parent_dir.joinpath(f"{NODE_NAME_PREFIX}_{i + 1}")
            target_dir = self.sdir.joinpath(f"{NODE_NAME_PREFIX}_{i + 1}")

            shutil.copy(source_dir.joinpath("./hoprd.id"), target_dir)
            shutil.copy(source_dir.joinpath("./.env"), target_dir)

    def reuse(self):
        logging.info("Re-using snapshot")

        # remove all files and folder in self.dir which are not snapshot and not
        # logs
        for entry in self.parent_dir.glob("*"):
            if entry.is_dir():
                continue
            elif not entry.name.endswith(".log") and not entry.name.endswith(".env"):
                logging.debug(f"Removing file: {entry}")
                entry.unlink(missing_ok=True)

        # copy snapshot files
        shutil.copytree(self.sdir, self.parent_dir, dirs_exist_ok=True)

        # remove log files from snapshot dir
        for entry in self.sdir.glob("*"):
            if entry.name.endswith(".log"):
                logging.debug(f"Removing log file from snapshot: {entry}")
                entry.unlink(missing_ok=True)

    @property
    def usable(self):
        expected_files = [
            self.sdir.joinpath(ANVIL_FOLDER_NAME),
            self.sdir.joinpath("barebone-lower-win-prob.cfg.yaml"),
            self.sdir.joinpath("barebone.cfg.yaml"),
            self.sdir.joinpath("default.cfg.yaml"),
        ]

        for f in expected_files:
            if not f.exists():
                logging.warning(f"Cannot find {f} in snapshot")
                return False

        return True

    @property
    def sdir(self):
        return self.parent_dir.joinpath(f"snapshot-{self.anvil_port}")
