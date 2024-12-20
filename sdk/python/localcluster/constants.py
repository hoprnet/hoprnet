import logging
from pathlib import Path

logging.basicConfig(format="%(asctime)s %(message)s")
logging.getLogger().setLevel(logging.INFO)

OPEN_CHANNEL_FUNDING_VALUE_HOPR = 1000
TICKET_PRICE_PER_HOP = 100

PWD = Path(__file__).parents[3]

IDENTITY_PREFIX = "hopr"
NODE_NAME_PREFIX = f"{IDENTITY_PREFIX}-node"

NETWORK = "anvil-localhost"
PASSWORD = "e2e-test"
PORT_BASE = 3000

SUITE_NAME = "hopr-localcluster"
MAIN_DIR = Path("/tmp").joinpath(SUITE_NAME)

ANVIL_FOLDER_NAME = "anvil"
ANVIL_FOLDER = MAIN_DIR.joinpath(ANVIL_FOLDER_NAME)

ANVIL_CONFIG_FILE = ANVIL_FOLDER.joinpath("anvil.cfg")

CONTRACTS_ADDRESSES = PWD.joinpath("ethereum", "contracts", "contracts-addresses.json")
