import logging
from pathlib import Path

logging.basicConfig(format="%(asctime)s %(message)s")
logging.getLogger().setLevel(logging.INFO)


OPEN_CHANNEL_FUNDING_VALUE_HOPR = 1000

TICKET_AGGREGATION_THRESHOLD = 100
TICKET_PRICE_PER_HOP = 100

RESERVED_TAG_UPPER_BOUND = 1023


FIXTURES_PREFIX = "hopr"
NODE_NAME_PREFIX = f"{FIXTURES_PREFIX}-node"

NETWORK1 = "anvil-localhost"

API_TOKEN = "e2e-API-token^^"
PASSWORD = "e2e-test"

PORT_BASE = 19000

SUITE_NAME = "hopr-localcluster"
ROOT_DIR = Path("/tmp")
MAIN_DIR = ROOT_DIR.joinpath(SUITE_NAME)

ANVIL_LOG_FILE = MAIN_DIR.joinpath("anvil.log")
ANVIL_STATE_FILE = MAIN_DIR.joinpath("anvil.state.json")
ANVIL_CONFIG_FILE = MAIN_DIR.joinpath("anvil.cfg")
PROTOCOL_CONFIG_FILE = MAIN_DIR.joinpath("protocol-config.json")

PWD = Path(__file__).parent.parent.parent

INPUT_PROTOCOL_CONFIG_FILE = PWD.parent.joinpath(
    "scripts", "protocol-config-anvil.json")
INPUT_DEPLOYMENTS_SUMMARY_FILE = PWD.parent.joinpath(
    "ethereum", "contracts", "contracts-addresses.json")
PREGENERATED_IDENTITIES_DIR = PWD.joinpath("identities")
