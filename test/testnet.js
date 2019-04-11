'use strict'

const { privKeyToPeerId } = require('../src/utils')
const {HARDCODED_PRIV_KEY } = require('../src/constants')

const FUNDING_KEY = HARDCODED_PRIV_KEY

const Ganache = require('ganache-core')

const server = Ganache.server({
    accounts: [
        {
            balance: '0xd3c21bcecceda0000000',
            secretKey: FUNDING_KEY
        }
    ]
})

server.listen(8545, 'localhost')