'use strict'

require('../config')

const chalk = require('chalk')

const fsPromise = require('fs').promises

const Web3 = require('web3')
const web3 = new Web3(process.env.PROVIDER)

const querystring = require('querystring')
const axios = require('axios')

const COMPILED_CONTRACTS_BASE_PATH = `${process.cwd()}/build/contracts`

const { deployContract } = require('../src/utils')

const IMPORT_SOLIDITY_REGEX = /^\s*import(\s+).*$/gm

async function main() {
    const index = await web3.eth.getTransactionCount(process.env.FUND_ACCOUNT_ETH_ADDRESS)
    await deployContract(index, web3)

    const contents = await fsPromise.readdir(COMPILED_CONTRACTS_BASE_PATH)

    const set = new Set()
    const paths = await Promise.all(
        contents.map(source => {
            const compiledContract = require(`${COMPILED_CONTRACTS_BASE_PATH}/${source}`)
            compiledContract.metadata = JSON.parse(compiledContract.metadata)

            const srcPaths = Object.keys(compiledContract.metadata.sources)

            return Promise.all(
                srcPaths.map(async srcPath => {
                    try {
                        await fsPromise.stat(srcPath)
                        return srcPath
                    } catch (err) {
                        try {
                            await fsPromise.stat(`${process.cwd()}/node_modules/${srcPath}`)
                            return `${process.cwd()}/node_modules/${srcPath}`
                        } catch (err) {
                            console.log(`Couldn't find import '${srcPath}'.`)
                        }
                    }
                })
            )
        })
    )

    paths.flat().forEach(path => set.add(path))

    const promises = []

    set.forEach(path => promises.push(fsPromise.readFile(path)))

    const sourceCode = (await Promise.all(promises)).map(source => source.toString().replace(IMPORT_SOLIDITY_REGEX, '')).join('\n')

    const metadata = require(`${process.cwd()}/build/contracts/HoprChannel.json`).metadata

    let subdomain = 'api'
    switch (process.env['NETWORK'].toLowerCase()) {
        case 'ropsten':
            subdomain += '-ropsten'
            break
        case 'rinkeby':
            subdomain += '-rinkeby'
            break
        default:
    }

    axios
        .post(
            `https://${subdomain}.etherscan.io/api`,
            querystring.stringify({
                apikey: process.env['ETHERSCAN_API_KEY'],
                module: 'contract',
                action: 'verifysourcecode',
                contractaddress: process.env[`CONTRACT_ADDRESS`],
                sourceCode,
                contractname: 'HoprChannel',
                compilerVersion: `v${metadata.compiler.version}`,
                constructorArguements: '',
                optimizationUsed: metadata.settings.optimizer.enabled ? '1' : '0',
                runs: metadata.settings.optimizer.runs.toString()
            })
        )
        .then(function(response) {
            console.log(response.statusText)
        })
        .catch(function(error) {
            console.log(error)
        })
}

main()