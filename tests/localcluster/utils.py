import asyncio
import json
import signal
from functools import wraps
from pathlib import Path


def coro(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

    return wrapper


def load_private_key(anvil_config: Path, pos=0):
    with open(anvil_config, "r") as file:
        data: dict = json.load(file)
        return data.get("private_keys", [""])[pos]


def wait_for_user_interrupt():
    print("Nodes are running. Press Ctrl+C to terminate.")
    try:
        signal.pause()
    except KeyboardInterrupt:
        pass
