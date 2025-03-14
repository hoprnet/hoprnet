import asyncio

import click

from . import bringup, utils


@click.command()
@click.option("--config", default="./localcluster.params.yml", help="Path to node config file")
@click.option(
    "--fully_connected",
    is_flag=True,
    show_default=True,
    default=False,
    help="Whether to open channels between all nodes.",
)
@click.option(
    "--docker_compose", is_flag=True, show_default=True, default=False, help="Whether to use the docker compose setup."
)
@utils.coro
async def main(config: str, fully_connected: bool, docker_compose: bool):
    cluster, anvil = await bringup(config, False, fully_connected, docker_compose)

    cluster.clean_up()
    if not docker_compose:
        anvil.kill()

    # delay to ensure that the cluster and anvil are down
    await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
