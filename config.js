'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

const myEnv = dotenv.config()
dotenvExpand(myEnv)

// Set contract address according to chosen network
process.env['CONTRACT_ADDRESS'] = process.env[`CONTRACT_ADDRESS_${process.env['NETWORK'].toUpperCase()}`]

// Set web3 provider info according to chosen network
process.env['PROVIDER'] = process.env[`PROVIDER_${process.env['NETWORK'].toUpperCase()}`]

const toDelete = Object.keys(process.env).filter(key => key.startsWith('CONTRACT_ADDRESS_') || key.startsWith('PROVIDER_'))

// Delete unused environment properties
toDelete.forEach(key => {
    delete process.env[key]
})
