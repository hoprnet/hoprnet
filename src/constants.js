'use strict'

const { toWei } = require('web3-utils')

const VERSION = '0.0.1'
const NAME = 'ipfs' // 'hopr'
const BASESTRING = `/${NAME}/${VERSION}`

const NETWORK = 'ropsten'
const contract = require('../config/contract-hopr.json');
const secrets = require('../config/.secrets.json')
const gas = require('../config/gasUnits.json');

module.exports = {
    DEMO: false,
    DEBUG: true,

    // possible options:
    // `ganache` => local Ganache testnet
    // `ropsten` => ropsten testnet
    // `rinkeby` => rinkeby testnet
    // `mainnet` => mainnet
    NETWORK: NETWORK,
    CONTRACT_ADDRESS: contract.contractAddress[NETWORK],

    CRAWLING_RESPONSE_NODES: 10,

    RELAY_FEE: toWei('100', 'wei'),

    PACKET_SIZE: 500,
    MAX_HOPS: 3,

    MARSHALLED_PUBLIC_KEY_SIZE: 37,

    NAME: NAME,

    PROTOCOL_STRING: `${BASESTRING}/msg`,
    PROTOCOL_ACKNOWLEDGEMENT: `${BASESTRING}/ack`,
    PROTOCOL_CRAWLING: `${BASESTRING}/crawl`,
    PROTOCOL_PAYMENT_CHANNEL: `${BASESTRING}/payment/open`,
    PROTOCOL_DELIVER_PUBKEY: `${BASESTRING}/pubKey`,
    PROTOCOL_SETTLE_CHANNEL: `${BASESTRING}/payment/settle`,
    PROTOCOL_STUN: `${BASESTRING}/stun`,
    PROTOCOL_HEARTBEAT: `${BASESTRING}/heartbeat`,
    PROTOCOL_WEBRTC_SIGNALING: `${BASESTRING}/webrtc/signaling`,
    PROTOCOL_WEBRTC_DATA: `${BASESTRING}/webrtc/data`,


    INFURA_URL: `https://${NETWORK}.infura.io/v3/${secrets.infuraApiKey}`,
    INFURA_WSS_URL: `wss://${NETWORK}.infura.io/ws/v3/${secrets.infuraApiKey}`,

    STAKE_GAS_AMOUNT: gas.stakeAmount,
    UNSTAKE_GAS_AMOUNT: gas.unstakeAmount,
    CREATE_GAS_AMOUNT: gas.createAmount,
    FUND_GAS_AMOUNT: gas.fundAmount,
    DEFAULT_GAS_AMOUNT: gas.defaultAmount,
    ETH_SEND_GAS_AMOUNT: gas.simpleTxAmount,
    GAS_PRICE: gas.gasPrice,

    HARDCODED_ETH_ADDRESS: secrets.fundAccountEthAddress,
    HARDCODED_PRIV_KEY: secrets.fundAccountPrivateKey
}