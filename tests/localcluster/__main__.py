import asyncio
import os
import random
import shutil
import signal

import click
import yaml

from . import utils
from .anvil import Anvil, AnvilState
from .cluster import Cluster
from .constants import (
    ANVIL_CONFIG_FILE,
    ANVIL_LOG_FILE,
    ANVIL_STATE_FILE,
    INPUT_DEPLOYMENTS_SUMMARY_FILE,
    MAIN_DIR,
    NETWORK1,
    NODE_NAME_PREFIX,
    PORT_BASE,
    PROTOCOL_CONFIG_FILE,
    logging,
)
from .snapshot import Snapshot

SNAPSHOT_FEATURE = False
SEED = int.from_bytes(os.urandom(8), byteorder="big")
random.seed(SEED)


def cleanup_data():
    # Remove old db
    for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}_*"):
        if not f.is_dir():
            continue
        logging.debug(f"Remove db in {f}")
        shutil.rmtree(f, ignore_errors=True)
    logging.info(f"Removed all dbs in {MAIN_DIR}")


@click.command()
@click.option("--config", default="./test_nodes.params.yml", help="Path to node config file")
@utils.coro
async def main(config: str):
    logging.info(f"Using the random seed: {SEED}")
    # load node config file
    with open(config, "r") as f:
        config = yaml.safe_load(f)

    cluster = Cluster(config, ANVIL_CONFIG_FILE, PROTOCOL_CONFIG_FILE)
    anvil = Anvil(ANVIL_LOG_FILE, ANVIL_CONFIG_FILE, ANVIL_STATE_FILE)

    if SNAPSHOT_FEATURE:
        snapshot = Snapshot(PORT_BASE, MAIN_DIR, cluster)

    # STOP OLD LOCAL ANVIL SERVER
    anvil.kill()

    cleanup_data()

    if not SNAPSHOT_FEATURE or not snapshot.usable:
        logging.info("Snapshot not usable")

        # START NEW LOCAL ANVIL SERVER
        anvil.run()
        anvil.mirror_contracts(
            INPUT_DEPLOYMENTS_SUMMARY_FILE,
            PROTOCOL_CONFIG_FILE,
            NETWORK1,
            NETWORK1,
        )

        # SETUP NODES USING STORED IDENTITIES
        cluster.copy_identities()
        cluster.create_safes()

        # wait before contract deployments are finalized
        await asyncio.sleep(2.5)

        # BRING UP NODES (with funding)
        await cluster.shared_bringup(skip_funding=False)

        if SNAPSHOT_FEATURE:
            snapshot.create(ANVIL_STATE_FILE)
    else:
        snapshot.reuse()

        anvil.run(AnvilState.LOAD)

        # SETUP NODES USING STORED IDENTITIES
        cluster.copy_identities()
        cluster.load_addresses()

        # wait before contract deployments are finalized
        await asyncio.sleep(2.5)

        # BRING UP NODES (without funding)
        await cluster.shared_bringup(skip_funding=True)

    # SHOW NODES' INFORMATIONS
    logging.info("All nodes ready")
    await cluster.links()

    try:
        utils.wait_for_user_interrupt()
    finally:
        # POST TEST CLEANUP
        logging.info(f"Tearing down the {cluster.size} nodes cluster")
        cluster.clean_up()
        anvil.kill()


if __name__ == "__main__":
    asyncio.run(main())
