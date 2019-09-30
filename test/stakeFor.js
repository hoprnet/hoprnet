'use strict'

require('../config')

const chalk = require('chalk')
const readline = require('readline')

const BN = require('bn.js')
const Web3 = require('web3')

const { toWei, toChecksumAddress } = require('web3-utils')

const { pubKeyToEthereumAddress, privKeyToPeerId } = require('../src/utils')
const { stakeFor, openChannelFor } = require('./utils')

function parseOptions() {
    const options = require('getopts')(process.argv.slice(2))

    const checkInput = input => typeof input === 'string' && input.replace(/0x/, '').length == 40
    if (!options._ || !Array.isArray(options._) || options._.length !== 2 || !checkInput(options._[0]) || !checkInput(options._[1]))
        throw Error(
            `Invalid beneficiaries. Expected two proper Ethereum addresses like ${chalk.green('0xe1c9D64858f37a954C871bA702B3F27F6E195685')} but got '${
                options._
            }'.`
        )

    const beneficiary0 = toChecksumAddress(options._[0])
    const beneficiary1 = toChecksumAddress(options._[1])

    if (!beneficiary0.startsWith('0x')) beneficiary0 = `0x${beneficiary0}`

    if (!beneficiary0.startsWith('0x')) beneficiary1 = `0x${beneficiary1}`

    return {
        beneficiary0,
        beneficiary1
    }
}
;(async () => {
    const { beneficiary0, beneficiary1 } = parseOptions()

    const fundingNode = await privKeyToPeerId(process.env['FUND_ACCOUNT_PRIVATE_KEY'])

    const web3 = new Web3(process.env.PROVIDER)

    const abi = require('../build/contracts/HoprChannel.json').abi
    const contract = new web3.eth.Contract(abi, process.env['CONTRACT_ADDRESS'], {
        from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
    })

    let [state0, state1, nonce] = await Promise.all([
        contract.methods.states(beneficiary0).call({
            from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
        }),
        contract.methods.states(beneficiary1).call({
            from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
        }),
        web3.eth.getTransactionCount(process.env.FUND_ACCOUNT_ETH_ADDRESS)
    ])

    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    })

    const batch = new web3.eth.BatchRequest()

    const handleStakeEther = (beneficiary, nonce) => {
        const promise = new Promise(resolve =>
            rl.question(
                `The account ${chalk.green(beneficiary)} seems to have no stakedEther. Do you want to stake some Ether for that party?, (${chalk.green(
                    'Y'
                )}/${chalk.red('n')}): `,
                answer => {
                    switch (answer.toLowerCase()) {
                        case 'y':
                            rl.question(`Amount (in ETH)? `, answer => {
                                batch.add(stakeFor(fundingNode, contract, nonce, beneficiary, toWei(answer, 'ether')))
                                resolve()
                            })
                            rl.write('0.1\n')
                            break
                        case 'n':
                        case '':
                        default:
                            return resolve()
                    }
                }
            )
        )

        rl.write('y\n')

        return promise
    }

    if (new BN(state0.stakedEther).isZero()) await handleStakeEther(beneficiary0, nonce++)

    if (new BN(state1.stakedEther).isZero()) await handleStakeEther(beneficiary1, nonce++)

    const query = openChannelFor(fundingNode, contract, nonce++, beneficiary0, beneficiary1)
    query.callback = (err) => {
        if (err) console.log(err.message)
    }

    batch.add(query)

    const result = await batch.execute()

    console.log('finished')

    return
})()
