'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

const myEnv = dotenv.config()
dotenvExpand(myEnv)

const readline = require('readline')

const chalk = require('chalk')
const rlp = require('rlp')

const groupBy = require('lodash.groupby')

const BN = require('bn.js')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const { toWei, fromWei } = require('web3-utils')

const Hopr = require('./src')
const { getId, pubKeyToEthereumAddress, pubKeyToPeerId, sendTransaction } = require('./src/utils')
const { STAKE_GAS_AMOUNT } = require('./src/constants')

const Transaction = require('./src/transaction')

const MINIMAL_STAKE = new BN(toWei('0.10', 'ether'))
const DEFAULT_STAKE = new BN(toWei('0.11', 'ether'))

let node, funds, ownAddress, stakedEther, rl, options

/**
 * Parses the given command-line options and returns a configuration object.
 *
 * @returns {Object}
 */
function parseOptions() {
    const options = require('getopts')(process.argv.slice(2), {
        alias: {
            b: 'bootstrap-node'
        }
    })

    if (Array.isArray(options._) && options._.length > 0) {
        options.id = Number.parseInt(options._[0])
    }

    const tmp = groupBy(process.env.BOOTSTRAP_SERVERS.split(',').map(addr => Multiaddr(addr)), ma => ma.getPeerId())
    options.bootstrapServers = Object.keys(tmp).reduce((acc, peerId) => {
        const peerInfo = new PeerInfo(PeerId.createFromB58String(peerId))

        tmp[peerId].forEach(ma => peerInfo.multiaddrs.add(ma))
        acc.push(peerInfo)
        return acc
    }, [])

    return options
}

/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param {PeerId} peerId
 * @returns {boolean}
 */
function isNotBootstrapNode(peerId) {
    return !node.bootstrapServers.some(peerInfo => peerInfo.id.isEqual(peerId))
}

/**
 * Get existing channels from the database
 *
 * @returns {Promise<PeerId[]>}
 */
function getExistingChannels() {
    return new Promise((resolve, reject) => {
        let promises = []

        node.db
            .createValueStream({
                gt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 0)),
                lt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 255))
            })
            .on('data', serializedTransaction => {
                const restoreTx = Transaction.fromBuffer(serializedTransaction)

                promises.push(pubKeyToPeerId(restoreTx.counterparty))
            })
            .on('end', () =>
                resolve(
                    Promise.all(promises).then(peerIds =>
                        peerIds.reduce((result, peerId) => {
                            if (isNotBootstrapNode) result.push(peerId)

                            return result
                        }, [])
                    )
                )
            )
            .on('error', reject)
    })
}

// Allowed keywords
const keywords = ['open', 'stake', 'stakedEther', 'unstake', 'send', 'quit', 'crawl', 'openChannels', 'closeAll']

/**
 * Completes a given input with possible endings. Used for convenience.
 *
 * @param {string} line the current input
 * @param {Function(Error, [String[], String])} cb called with `(err, [possibleCompletions, currentLine])`
 */
function tabCompletion(line, cb) {
    const operands = line.trim().split(' ')

    let hits
    switch (operands[0].toLowerCase()) {
        case 'send':
            hits = node.peerBook.getAllArray().filter(peerInfo => {
                if (operands.length > 1 && !peerInfo.id.toB58String().startsWith(operands[1])) return false

                return isNotBootstrapNode(peerInfo.id)
            })

            if (!hits.length) {
                console.log(chalk.red(`\nDoesn't know any other node except the bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}!`))
                return cb(null, [[''], line])
            }

            return cb(null, [hits.map(peerInfo => `send ${peerInfo.id.toB58String()}`), line])
        case 'stake':
            if (funds.isZero()) {
                console.log(chalk.red(`\nCan't stake any funds without any Ether.`))
                return [['stake 0.00'], line]
            }

            return cb(null, [[`stake ${fromWei(funds)}`], line])

        case 'unstake':
            return cb(null, [[`unstake ${fromWei(stakedEther, 'ether')}`], line])
        case 'open':
            getExistingChannels()
                .then(peerIds => {
                    const peers = node.peerBook.getAllArray().reduce((result, peerInfo) => {
                        if (isNotBootstrapNode(peerInfo.id) && !peerIds.some(peerId => peerId.isEqual(peerInfo.id))) result.push(peerInfo.id.toB58String())

                        return result
                    }, [])

                    if (peers.length < 1) {
                        console.log(chalk.red(`\nDoesn't know any node to open a payment channel with.`))
                        return cb(null, [[''], line])
                    }

                    hits = operands.length > 1 ? peers.filter(peerId => peerId.startsWith(operands[1])) : peers

                    return cb(null, [hits.length ? hits.map(str => `open ${str}`) : ['open'], line])
                })
                .catch(err => {
                    console.log(chalk.red(err.message))
                    return cb(null, [[''], line])
                })
            break
        case 'close':
            getExistingChannels()
                .then(peerIds => {
                    if (peerIds.length < 1) {
                        console.log(chalk.red(`\nCan't close a payment channel as there aren't any open ones!`))
                        return cb(null, [[''], line])
                    }

                    hits =
                        operands > 1
                            ? peerIds.reduce((result, peerId) => {
                                  if (peerId.toB58String().startsWith(operands[1])) result.push(peerId.toB58String())

                                  return result
                              }, [])
                            : peerIds.map(peerId => peerId.toB58String())

                    return cb(null, [hits.length ? hits.map(str => `close ${str}`) : ['close'], line])
                })
                .catch(err => {
                    console.log(chalk.red(err.message))
                    return cb(null, [[''], line])
                })
            break
        default:
            hits = keywords.filter(keyword => keyword.startsWith(line))

            return cb(null, [hits.length ? hits : keywords, line])
    }
}

/**
 * Locks the given amount in the smart contract.
 *
 * @param {BN | string} amount
 */
function stakeEther(amount) {
    if (!amount) amount = DEFAULT_STAKE

    return sendTransaction(
        {
            from: ownAddress,
            to: process.env.CONTRACT_ADDRESS,
            value: amount,
            gas: STAKE_GAS_AMOUNT
        },
        node.peerInfo.id,
        node.paymentChannels.web3
    ).then(() => {
        node.paymentChannels.nonce = node.paymentChannels.nonce + 1
    })
}

function stopNode() {
    const timeout = setTimeout(() => {
        console.log(`Ungracefully stopping node after timeout.`)
        process.exit(0)
    }, 10 * 1000)

    node.stop()
        /* prettier-ignore */
        .then(() => clearTimeout(timeout))
        .catch(() => process.exit(0))
}

function runAsBootstrapNode() {
    if (options['bootstrap-node']) console.log(`... running as bootstrap node!.`)

    node.on('peer:connect', peer => {
        console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
    })
}

async function runAsRegularNode() {
    ownAddress = pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())

    try {
        ;[funds, stakedEther] = await Promise.all([
            node.paymentChannels.web3.eth.getBalance(ownAddress).then(result => new BN(result)),
            node.paymentChannels.contract.methods
                .states(ownAddress)
                .call({ from: ownAddress })
                .then(result => new BN(result.stakedEther))
        ])
    } catch (err) {
        console.log(chalk.red(err.message))
        return stopNode()
    }

    console.log(
        `Own Ethereum address: ${chalk.green(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()))}\n` +
            `Funds: ${fromWei(funds, 'ether')} ETH\n` +
            `Stake: ${fromWei(stakedEther, 'ether')} ETH\n`
    )

    if (stakedEther.add(funds).lt(MINIMAL_STAKE)) {
        console.log(
            chalk.red(
                `Insufficient funds. Got only ${fromWei(stakedEther.add(funds), 'ether')} ETH to stake. Please fund the account ${chalk.green(
                    ownAddress.toString()
                )} with at least ${fromWei(MINIMAL_STAKE.sub(funds).sub(stakedEther), 'Ether')} ETH.`
            )
        )
        return stopNode()
    }

    if (stakedEther.lt(MINIMAL_STAKE)) {
        await new Promise((resolve, reject) =>
            rl.question(
                `Staked Ether is less than ${fromWei(MINIMAL_STAKE, 'ether')} ETH. Do you want to refund now? (${chalk.green('Y')}/${chalk.red('n')}): `,
                answer => {
                    switch (answer.toLowerCase()) {
                        case '':
                        case 'y':
                            rl.question(`Amount? : `, answer => {
                                resolve(stakeEther(toWei(answer)))
                            })
                            rl.write(fromWei(MINIMAL_STAKE.sub(stakedEther), 'ether'))
                            break
                        default:
                            return stopNode()
                    }
                }
            )
        )
    }

    console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

    // Scan restore transactions and detect whether they're on-chain available

    rl.on('line', input => {
        rl.pause()
        const operands = input.trim().split(' ')
        let amount
        switch (operands[0]) {
            case 'crawl':
                node.crawler
                    .crawl(peerInfo => isNotBootstrapNode(peerInfo.id))
                    .catch(err => console.log(chalk.red(err.message)))
                    .finally(() => {
                        setTimeout(() => {
                            readline.clearLine(process.stdin, 0)
                            rl.prompt()
                        })
                    })
                break
            case 'quit':
                stopNode()
                break
            case 'stake':
                if (operands.length != 2) {
                    console.log(chalk.red(`Invalid arguments. Expected 'stake <amount of ETH>'. Received '${input}'`))
                    rl.prompt()
                    break
                }
                amount = new BN(toWei(operands[1], 'ether'))
                if (funds.lt(new BN(amount))) {
                    console.log(chalk.red('Insufficient funds.'))
                    rl.prompt()
                    break
                }

                stakeEther(amount).then(() => {
                    stakedEther.iadd(amount)
                    funds.isub(amount)

                    rl.prompt()
                })
                break
            case 'stakedEther':
                node.paymentChannels.contract.methods
                    .states(ownAddress)
                    .call({ from: ownAddress })
                    .then(state => {
                        stakedEther = new BN(state.stakedEther)
                        console.log(`Current stake: ${chalk.green(fromWei(state.stakedEther, 'ether'))} ETH`)
                        rl.prompt()
                    })
                break
            case 'unstake':
                if (operands.length != 2) {
                    console.log(chalk.red(`Invalid arguments. Expected 'unstake <amount of ETH>'. Received '${input}'`))
                    rl.prompt()
                    break
                }
                amount = new BN(toWei(operands[1], 'ether'))
                if (stakedEther.lt(amount)) {
                    console.log(chalk.red('Amount must not be higher than current stake.'))
                    rl.prompt()
                    break
                }

                node.paymentChannels
                    .contractCall(node.paymentChannels.contract.methods.unstakeEther(amount.toString()))
                    .then(() => rl.prompt())
                    .catch(err => {
                        console.log(err)
                        rl.prompt()
                    })
                break
            case 'openChannels':
                let str = `${chalk.yellow('ChannelId:'.padEnd(64, ' '))} - ${chalk.blue('PeerId:')}`
                let index = 0
                let promises = []
                node.db
                    .createReadStream({
                        gt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 0)),
                        lt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 255))
                    })
                    .on('data', ({ key, value }) => {
                        index++
                        const channelId = key.slice(key.length - 32)
                        const restoreTx = Transaction.fromBuffer(value)

                        promises.push(
                            pubKeyToPeerId(restoreTx.counterparty).then(peerId => {
                                str += `\n${chalk.yellow(channelId.toString('hex'))} - ${chalk.blue(peerId.toB58String())}`
                            })
                        )
                    })
                    .on('end', () => {
                        if (index == 0) {
                            str += `\n  No open channels.`
                            console.log(str)
                            rl.prompt()
                        } else {
                            Promise.all(promises).then(() => {
                                console.log(str)
                                rl.prompt()
                            })
                        }
                    })
                break
            case 'open':
                if (operands.length != 2) {
                    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                let peerInfo
                try {
                    peerInfo = node.peerBook.get(PeerId.createFromB58String(operands[1]))
                } catch (err) {
                    console.log(chalk.red('Unable to open payment channel.'))
                    rl.prompt()
                }

                const channelId = getId(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal()))

                let interval
                node.paymentChannels
                    .open(peerInfo.id)
                    .then(() => {
                        console.log(`${chalk.green(`Successfully opened channel`)} ${chalk.yellow(channelId.toString('hex'))}`)
                    })
                    .catch(err => {
                        console.log(chalk.red(err.message))
                    })
                    .finally(() => {
                        clearInterval(interval)
                        setTimeout(() => {
                            readline.clearLine(process.stdin, 0)
                            rl.prompt()
                        })
                    })

                process.stdout.write(`Submitted transaction. Waiting for confirmation .`)
                interval = setInterval(() => process.stdout.write('.'), 1000)
                break
            case 'send':
                if (operands.length != 2) {
                    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                rl.question(`Sending message to ${chalk.blue(operands[1])}\nType in your message and press ENTER to send:\n`, message =>
                    node
                        .sendMessage(rlp.encode([message, Date.now().toString()]), operands[1])
                        .catch(err => console.log(chalk.red(err.message)))
                        .finally(() => rl.prompt())
                )
                break
            case 'closeAll':
                node.paymentChannels
                    .closeChannels()
                    .then(receivedMoney => {
                        console.log(`${chalk.green(`Closed all channels and received`)} ${chalk.magenta(fromWei(receivedMoney.toString(), 'ether'))} ETH.`)
                        rl.prompt()
                    })
                    .catch(err => {
                        console.log(err)
                        rl.prompt()
                    })
                break
            case 'close':
                if (operands.length != 2) {
                    console.log(chalk.red(`Invalid arguments. Expected 'close <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                try {
                    const peerInfo = node.peerBook.get(PeerId.createFromB58String(operands[1]))
                    const channelId = getId(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal()))

                    let interval
                    node.paymentChannels
                        .closeChannel(channelId)
                        .then(receivedMoney => {
                            console.log(
                                `${chalk.green(`Successfully closed channel`)} ${chalk.yellow(channelId.toString('hex'))}. Received ${chalk.magenta(
                                    fromWei(receivedMoney, 'ether')
                                )} ETH.`
                            )
                            clearInterval(interval)
                            setTimeout(() => {
                                readline.clearLine(process.stdin, 0)
                                rl.prompt()
                            })
                        })
                        .catch(err => {
                            console.log(err.message)
                            clearInterval(interval)
                            readline.clearLine(process.stdin, 0)
                            rl.prompt()
                        })
                    console.log(`Submitted transaction. Waiting for confirmation .`)
                    interval = setInterval(() => process.stdout.write('.'), 1000)
                } catch (err) {
                    console.log(chalk.red(err.message))
                    rl.prompt()
                }
                break

            default:
                console.log(chalk.red('Unknown command!'))
                rl.prompt()
        }
    })
    rl.prompt()
}

process.title = 'hopr'

async function main() {
    console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

    options = parseOptions()

    try {
        node = await Hopr.createNode(options)
    } catch (err) {
        console.log(chalk.red(err.message))
        process.exit(1)
    }

    console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)

    rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
        completer: tabCompletion
    })

    rl.on('close', stopNode)

    if (options['bootstrap-node']) {
        runAsBootstrapNode()
    } else {
        runAsRegularNode()
    }
}

main()
