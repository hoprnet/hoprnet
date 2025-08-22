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
    ANVIL_FOLDER,
    BASE_PORT,
    CONTRACTS_ADDRESSES,
    MAIN_DIR,
    NETWORK,
    logging,
)
from .snapshot import Snapshot

SEED = int.from_bytes(os.urandom(8), byteorder="big")
random.seed(SEED)


async def bringup(
    config: str,
    size: Optional[int] = None,
    test_mode: bool = False,
    fully_connected: bool = False,
    use_nat: bool = False,
    exposed: bool = False,
    base_port: int = BASE_PORT,
) -> Optional[Tuple[Cluster, Anvil]]:
    logging.info(f"Using the random seed: {SEED}")

    if test_mode and fully_connected:
        logging.warning("`fully_connected` feature is not compatible with test mode. Ignore it.")

    # load node config file
    with open(config, "r") as f:
        config = yaml.safe_load(f)

    cluster = Cluster(
        config, ANVIL_CONFIG_FILE, ANVIL_FOLDER.joinpath("protocol-config.json"), use_nat, exposed, base_port, size
    )
    anvil = Anvil(
        ANVIL_FOLDER.joinpath("anvil.log"),
        ANVIL_CONFIG_FILE,
        ANVIL_FOLDER.joinpath("anvil.state.json"),
        base_port,
        not test_mode,
    )

    snapshot = Snapshot(base_port, MAIN_DIR, cluster)

    # STOP OLD LOCAL ANVIL SERVER
    anvil.kill()

    # Remove old logs
    for f in MAIN_DIR.glob("*/*.log"):
        logging.debug(f"Removing log file: {f}")
        f.unlink(missing_ok=True)

    if not snapshot.usable:
        logging.info("Snapshot not usable")

        # START NEW LOCAL ANVIL SERVER
        anvil.run()
        anvil.mirror_contracts(
            CONTRACTS_ADDRESSES,
            ANVIL_FOLDER.joinpath("protocol-config.json"),
            NETWORK,
            NETWORK,
        )

        # SETUP NODES USING STORED IDENTITIES
        cluster.copy_identities()
        cluster.create_safes()

        # wait before contract deployments are finalized
        await asyncio.sleep(2.5)

        # enable network registry
        cluster.enable_network_registry()
        await asyncio.sleep(1)

        # fund nodes
        cluster.fund_nodes()

        # delay to ensure anvil is stopped and state file closed
        anvil.kill()
        await asyncio.sleep(1)

        snapshot.create()

    snapshot.reuse()

    anvil.run(AnvilState.LOAD)

    # SETUP NODES USING STORED IDENTITIES
    cluster.copy_identities()
    cluster.load_addresses()
    cluster.load_native_addresses()

    # wait before contract deployments are finalized
    await asyncio.sleep(2.5)

    # BRING UP NODES
    try:
        await cluster.shared_bringup()
    except asyncio.TimeoutError as e:
        logging.error(f"Timeout error: {e}")
        return cluster, anvil
    except RuntimeError as e:
        cluster.clean_up()
        anvil.kill()
        raise e

    if not test_mode and fully_connected:
        await cluster.connect_peers()

    logging.info("All nodes ready")

    if not test_mode:
        # SHOW NODES' INFORMATIONS
        await cluster.links()

        try:
            utils.wait_for_user_interrupt()
        finally:
            pass

    return cluster, anvil
