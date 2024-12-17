import asyncio

import click

from . import bringup
from .utils import coro


@click.command()
@click.option("--config", default="./localcluster.params.yml", help="Path to node config file")
@coro
async def main(config: str):
    await bringup(config, test_mode=False)

if __name__ == "__main__":
    asyncio.run(main())
