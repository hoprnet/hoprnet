import asyncio

import click

from . import bringup, utils


@click.command()
@click.option("--config", default="./localcluster.params.yml", help="Path to node config file")
@click.option(
    "--fully_connected", is_flag=True, show_default=True, default=False, help="Creates channel between all nodes"
)
@click.option("--exposed", is_flag=True, show_default=True, default=False, help="Expose the nodes to the local network")
@utils.coro
async def main(config: str, fully_connected: bool, exposed: bool):
    cluster, anvil = await bringup(config, False, fully_connected, exposed=exposed)

    cluster.clean_up()
    anvil.kill()

    # delay to ensure that the cluster and anvil are down
    await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
