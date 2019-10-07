'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

const myEnv = dotenv.config()
dotenvExpand(myEnv)

const { toWei } = require('web3-utils')

// Set contract address according to chosen network
process.env['CONTRACT_ADDRESS'] = process.env[`CONTRACT_ADDRESS_${process.env['NETWORK'].toUpperCase()}`]

// Set web3 provider info according to chosen network
process.env['PROVIDER'] = process.env[`PROVIDER_${process.env['NETWORK'].toUpperCase()}`]

// Parse and set gas price
const gasPrice = process.env[`GAS_PRICE_${process.env['NETWORK'].toUpperCase()}`].trim()
if (!gasPrice)
    throw Error(`Please set variable 'GAS_PRICE_${process.env['NETWORK'].toUpperCase()}' in your '.env' file.`)

const [amount, unit] = gasPrice.split(' ')
process.env['GAS_PRICE'] = toWei(amount, unit.toLowerCase())

const toDelete = Object.keys(process.env).filter(key => key.startsWith('CONTRACT_ADDRESS_') || key.startsWith('PROVIDER_') || key.startsWith('GAS_PRICE_'))

// Delete unused environment properties
toDelete.forEach(key => {
    delete process.env[key]
})
