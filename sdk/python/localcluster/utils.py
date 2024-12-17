import asyncio
import functools
import json
import logging
import pathlib
import shutil
import signal

from .constants import MAIN_DIR, NODE_NAME_PREFIX


def coro(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        return asyncio.run(f(*args, **kwargs))

    return wrapper


def load_private_key(anvil_config: pathlib.Path, pos=0):
    with open(anvil_config, "r") as file:
        data: dict = json.load(file)
        return data.get("private_keys", [""])[pos]


def wait_for_user_interrupt():
    print("Nodes are running. Press Ctrl+C to terminate.")
    try:
        signal.pause()
    except KeyboardInterrupt:
        pass


def cleanup_data():
    # Remove old db
    for f in MAIN_DIR.glob(f"{NODE_NAME_PREFIX}_*"):
        if not f.is_dir():
            continue
        logging.debug(f"Remove db in {f}")
        shutil.rmtree(f, ignore_errors=True)
    logging.info(f"Removed all dbs in {MAIN_DIR}")
