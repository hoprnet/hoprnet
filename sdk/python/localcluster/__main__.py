import asyncio

import click

from . import bringup, utils


@click.command()
@click.option("--config", default="./localcluster.params.yml", help="Path to node config file")
@click.option(
    "--fully_connected", is_flag=True, show_default=True, default=False, help="Creates channel between all nodes"
)
@click.option("--exposed", is_flag=True, show_default=True, default=False, help="Expose the nodes to the local network")
@click.option("--size", default=6, show_default=True, help="Number of nodes in the cluster")
@utils.coro
async def main(config: str, fully_connected: bool, exposed: bool, size: int):
    cluster_and_anvil = await bringup(
        config, test_mode=False, fully_connected=fully_connected, exposed=exposed, size=size
    )

    assert cluster_and_anvil is not None, "Failed to bring up the cluster"
    cluster, anvil = cluster_and_anvil

    cluster.clean_up()
    anvil.kill()

    # delay to ensure that the cluster and anvil are down
    await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
