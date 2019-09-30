'use strict'

require('../config')


const COMPILED_CONTRACTS_BASE_PATH = `${process.cwd()}/build/contracts`

const Web3 = require('web3')

const { deployContract } = require('../src/utils')
const { wait, verifyContractCode } = require('./utils')

;(async function main() {
    const web3 = new Web3(process.env['PROVIDER'])
    const nonce = await web3.eth.getTransactionCount(process.env['FUND_ACCOUNT_ETH_ADDRESS'])
    const contractAddress = await deployContract(nonce, web3)

    if (process.env['NETWORK'] === 'ganache') return

    console.log(`Giving the testnet 30 seconds to propagate the contract code before trying to verify the contract.`)
    await wait(30 * 1000)

    return verifyContractCode(COMPILED_CONTRACTS_BASE_PATH, contractAddress)
})()
