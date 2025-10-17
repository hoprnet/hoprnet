import logging
from pathlib import Path
import os

from sdk.python.api.balance import Balance

logging.basicConfig(format="%(asctime)s %(message)s")
# Read desired log level from environment variable (default: INFO)
log_level = os.getenv("LOG_LEVEL", "INFO").upper()
logging.getLogger().setLevel(log_level)

OPEN_CHANNEL_FUNDING_VALUE_HOPR = Balance("1000 wxHOPR")
TICKET_PRICE_PER_HOP = Balance("100 wxHOPR")

PWD = Path(__file__).parents[3]

IDENTITY_PREFIX = "hopr"
NODE_NAME_PREFIX = f"{IDENTITY_PREFIX}-node"

NETWORK = "anvil-localhost"
PASSWORD = "e2e-test"
BASE_PORT = 3000

SUITE_NAME = "hopr-localcluster"
MAIN_DIR = Path("/tmp").joinpath(SUITE_NAME)
CONTRACTS_DIR = PWD.joinpath("ethereum/contracts")

ANVIL_FOLDER_NAME = "anvil"
ANVIL_FOLDER = MAIN_DIR.joinpath(ANVIL_FOLDER_NAME)

ANVIL_CONFIG_FILE = ANVIL_FOLDER.joinpath("anvil.cfg")

CONTRACTS_ADDRESSES = CONTRACTS_DIR.joinpath("contracts-addresses.json")
