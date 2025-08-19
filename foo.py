import asyncio
import pprint

import yaml
from api_lib.headers.authorization import Bearer

from sdk.python.api.hopr import HoprdAPI
from sdk.python.api.yaml_parser import PlainTextLoader


async def main():
    api = HoprdAPI("http://localhost:3003", Bearer("e2e-API-token^^"), "/api/v4")

    config = await api.config()

    strategies_field = {k: v for d in config.strategies for k, v in d.items()}

    print(f"{strategies_field["unrealized_balance_ratio"]=}")


if __name__ == "__main__":
    asyncio.run(main())
