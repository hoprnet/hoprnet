'use strict'

const { privKeyToPeerId, compileIfNecessary } = require('../src/utils')

const { NET, GAS_PRICE, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const FUNDING_ACCOUNT = HARDCODED_ETH_ADDRESS
const FUNDING_KEY = HARDCODED_PRIV_KEY

const Ganache = require('ganache-core')


let index, compiledContract


const server = Ganache.server({
    accounts: [
        {
            balance: '0xd3c21bcecceda0000000',
            secretKey: FUNDING_KEY
        }
    ]
})

server.listen(8545, 'localhost')



const fundingPeer = privKeyToPeerId(FUNDING_KEY)