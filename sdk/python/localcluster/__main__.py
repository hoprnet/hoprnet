import asyncio
import logging

import click

from . import bringup, utils


@click.command()
@click.option("--config", default="./localcluster.params.yml", help="Path to node config file")
@click.option(
    "--fully_connected", is_flag=True, show_default=True, default=False, help="Creates channel between all nodes"
)
@click.option("--exposed", is_flag=True, show_default=True, default=False, help="Expose the nodes to the local network")
@click.option("--size", default=6, show_default=True, help="Number of nodes in the cluster")
@click.option(
    "--background", is_flag=True, show_default=True, default=False, help="Run in background at the end of setup"
)
@utils.coro
async def main(config: str, fully_connected: bool, exposed: bool, size: int, background: bool):
    test_mode: bool = False

    if background:
        logging.warning("Nodes and anvil will be running in the background after setup")
    if test_mode:
        logging.warning("Running in test mode")

    cluster_and_anvil = await bringup(
        config, test_mode=test_mode, fully_connected=fully_connected, exposed=exposed, size=size
    )

    assert cluster_and_anvil is not None, "Failed to bring up the cluster"
    cluster, anvil = cluster_and_anvil

    if not test_mode:
        await cluster.links()

        if not background:
            try:
                utils.wait_for_user_interrupt()
            finally:
                pass
        else:
            logging.info(f"Anvil Process ID: {anvil.process_id}")

    if not background:
        cluster.clean_up()
        anvil.kill()

        # delay to ensure that the cluster and anvil are down
        await asyncio.sleep(1)

    return cluster, anvil


if __name__ == "__main__":
    asyncio.run(main())
