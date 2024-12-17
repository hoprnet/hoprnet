import asyncio
import os
import random
from typing import Optional, Tuple

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
    PORT_BASE,
    PROTOCOL_CONFIG_FILE,
    logging,
)
from .snapshot import Snapshot

SNAPSHOT_FEATURE = False
SEED = int.from_bytes(os.urandom(8), byteorder="big")
random.seed(SEED)


async def bringup(config: str, test_mode: bool = False) -> Optional[Tuple[Cluster, Anvil]]:
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

    utils.cleanup_data()

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

    if not test_mode:
        await cluster.links()

        try:
            utils.wait_for_user_interrupt()
        finally:
            cluster.clean_up()
            anvil.kill()
    else:
        return cluster, anvil
