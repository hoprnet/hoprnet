'use strict'

module.exports.PACKET_SIZE = 500

module.exports.MAX_HOPS = 3

module.exports.PROTOCOL_VERSION = '0.0.1'

module.exports.PROTOCOL_NAME = 'ipfs'
module.exports.PROTOCOL_STRING = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/msg').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_ACKNOWLEDGEMENT = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/acknowledgement/').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_CRAWLING = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/crawl').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_PAYMENT_CHANNEL = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/paymentChannel').concat(this.PROTOCOL_VERSION)

module.exports.PROTOCOL_DELIVER_PUBKEY = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/pubKey').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_SETTLE_CHANNEL = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/settleChannel').concat(this.PROTOCOL_VERSION)

module.exports.MARSHALLED_PUBLIC_KEY_SIZE = 37

module.exports.CRAWLING_RESPONSE_NODES = 10

module.exports.RELAY_FEE = '100' // Wei

module.exports.DEMO = false
module.exports.DEBUG = true

module.exports.NET = 'ganache'

// HoprChannel Contract Configuration TODO: detect ENV variable to decide between mainnet or testnet
const contract = require('../config/contract-hopr.json');
module.exports.CONTRACT_ADDRESS = contract.ropstenContractAddress;

// Gas Unit Configuration TODO: get all safe upper bound gas amounts
const gas = require('../config/gasUnits.json');
module.exports.STAKE_GAS_AMOUNT = gas.stakeAmount;
module.exports.UNSTAKE_GAS_AMOUNT = gas.unstakeAmount;
module.exports.CREATE_GAS_AMOUNT = gas.createAmount;
module.exports.FUND_GAS_AMOUNT = gas.fundAmount;
module.exports.DEFAULT_GAS_AMOUNT = gas.defaultAmount;
module.exports.ETH_SEND_GAS_AMOUNT = gas.simpleTxAmount;
module.exports.GAS_PRICE = gas.gasPrice;

const secrets = require('../config/.secrets.json');
// Infura API Configuration
module.exports.INFURA_KEY = secrets.infuraApiKey;
module.exports.ROPSTEN_URL = secrets.infuraRopstenURL + secrets.infuraApiKey;
module.exports.ROPSTEN_WSS_URL = secrets.infuraRopstenWssURL + secrets.infuraApiKey;

module.exports.MAINNET_URL = secrets.infuraMainnetURL + secrets.infuraApiKey;
module.exports.MAINNET_WSS_URL = secrets.infuraMainnetWssURL + secrets.infuraApiKey;

// Private Key Configuration
module.exports.HARDCODED_ETH_ADDRESS = secrets.fundAccountEthAddress;
module.exports.HARDCODED_PRIV_KEY = secrets.fundAccountPrivateKey;
