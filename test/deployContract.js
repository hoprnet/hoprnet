'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

var myEnv = dotenv.config()
dotenvExpand(myEnv)

const Web3 = require('web3')
const web3 = new Web3(process.env.PROVIDER)

const { deployContract } = require('../src/utils')

async function main() {
    const index = await web3.eth.getTransactionCount(process.env.FUND_ACCOUNT_ETH_ADDRESS)
    await deployContract(index, web3)
}

main()
