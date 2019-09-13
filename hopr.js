'use strict'

require('./config')

const readline = require('readline')

const chalk = require('chalk')
const rlp = require('rlp')

const groupBy = require('lodash.groupby')

const BN = require('bn.js')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const Multihash = require('multihashes')
const bs58 = require('bs58')

const { toWei, fromWei } = require('web3-utils')

const Hopr = require('./src')
const { getId, pubKeyToEthereumAddress, pubKeyToPeerId, sendTransaction, addPubKey } = require('./src/utils')
const { STAKE_GAS_AMOUNT } = require('./src/constants')

const Transaction = require('./src/transaction')

const MINIMAL_STAKE = new BN(toWei('0.10', 'ether'))
const DEFAULT_STAKE = new BN(toWei('0.11', 'ether'))

const HASH_LENGTH = 32
const CHANNEL_ID_LENGTH = HASH_LENGTH

const SPLIT_OPERAND_QUERY_REGEX = /([\w\-]+)(?:\s+)?([\w\s\-]+)?/

let node, funds, ownAddress, stakedEther, rl, options

/**
 * Parses the given command-line options and returns a configuration object.
 *
 * @returns {Object}
 */
function parseOptions() {
    const displayHelp = () => {
        console.log(
            /* prettier-ignore */
            `\nStart HOPR with:\n` +
            `-b [--bootstrap-node, bootstrap]\tstarts HOPR as a bootstrap node\n` +
            `<ID>\t\t\t\t\tstarts HOPR with ID <ID> specified .env\n`
        )
        process.exit(0)
    }

    const unknownOptions = []

    const options = require('getopts')(process.argv.slice(2), {
        boolean: ['bootstrap-node'],
        alias: {
            b: 'bootstrap-node'
        },
        unknown: option => unknownOptions.push(option)
    })

    if (Array.isArray(options._)) {
        options._.forEach(option => {
            if (option.toLowerCase() === 'bootstrap') {
                options['bootstrap-node'] = true
                return
            }

            const int = parseInt(option)
            if (isFinite(int)) {
                if (options.id) {
                    console.log(`Cannot set ID twice.`)
                    return
                }

                options.id = int
                return
            }

            unknownOptions.push(option)
        })
    }

    if (unknownOptions.length) {
        console.log(`Got unknown option${unknownOptions.length == 1 ? '' : 's'} [${unknownOptions.join(', ')}]`)
        return displayHelp()
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
    return node.paymentChannels.getAllChannels(
        channel => pubKeyToPeerId(channel.state.restoreTransaction.counterparty),
        promises => Promise.all(promises).then(peerIds => peerIds.filter(isNotBootstrapNode))
    )
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
    const [command, query] = line
        .trim()
        .split(SPLIT_OPERAND_QUERY_REGEX)
        .slice(1)

    let hits
    switch (command) {
        case 'send':
            hits = node.peerBook.getAllArray().filter(peerInfo => {
                if (query && !peerInfo.id.toB58String().startsWith(query)) return false

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

                    hits = query ? peers.filter(peerId => peerId.startsWith(query)) : peers

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
                    if (peerIds && peerIds.length < 1) {
                        console.log(chalk.red(`\nCan't close a payment channel as there aren't any open ones!`))
                        return cb(null, [[''], line])
                    }

                    hits = query
                        ? peerIds.reduce((result, peerId) => {
                              if (peerId.toB58String().startsWith(query)) result.push(peerId.toB58String())

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
    ).then(receipt => {
        node.paymentChannels.nonce = node.paymentChannels.nonce + 1
        return receipt
    })
}

/**
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param {string} query query that contains the peerId
 */
async function checkPeerIdInput(query) {
    let peerId

    try {
        // Throws an error if the Id is invalid
        Multihash.decode(bs58.decode(query))

        peerId = await addPubKey(PeerId.createFromB58String(query))
    } catch (err) {
        throw Error(chalk.red(`Invalid peerId. ${err.message}`))
    }
    return peerId
}

/**
 * Stops the node and kills the process in case it does not quit by itself.
 */
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
                `Staked Ether is less than ${fromWei(MINIMAL_STAKE, 'ether')} ETH. Do you want to increase the stake now? (${chalk.green('Y')}/${chalk.red(
                    'n'
                )}): `,
                answer => {
                    switch (answer.toLowerCase()) {
                        case '':
                        case 'y':
                            rl.question(`Amount? : `, answer => resolve(stakeEther(toWei(answer))))
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

    rl.on('line', async input => {
        rl.pause()
        const [command, query] = (input || '')
            .trim()
            .split(SPLIT_OPERAND_QUERY_REGEX)
            .slice(1)

        let amount, peerId
        switch ((command || '').trim()) {
            case 'crawl':
                try {
                    await node.crawler.crawl(peerInfo => isNotBootstrapNode(peerInfo.id))
                } catch (err) {
                    console.log(chalk.red(err.message))
                } finally {
                    setTimeout(() => {
                        readline.clearLine(process.stdin, 0)
                        rl.prompt()
                    })
                }
                break
            case 'quit':
                stopNode()
                break
            case 'stake':
                if (!query) {
                    console.log(chalk.red(`Invalid arguments. Expected 'stake <amount of ETH>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                amount = new BN(toWei(query, 'ether'))
                if (funds.lt(new BN(amount))) {
                    console.log(chalk.red('Insufficient funds.'))
                    rl.prompt()
                    break
                }

                try {
                    await stakeEther(amount)

                    stakedEther.iadd(amount)
                    funds.isub(amount)
                } catch (err) {
                    console.log(chalk.red(err.message))
                } finally {
                    setTimeout(() => {
                        rl.prompt()
                    })
                }
                break
            case 'stakedEther':
                try {
                    let state = await node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress })
                    stakedEther = new BN(state.stakedEther)
                    console.log(`Current stake: ${chalk.green(fromWei(state.stakedEther, 'ether'))} ETH`)
                } catch (err) {
                    console.log(chalk.red(err.message))
                } finally {
                    setTimeout(() => {
                        rl.prompt()
                    })
                }
                break
            case 'unstake':
                if (!query) {
                    console.log(chalk.red(`Invalid arguments. Expected 'unstake <amount of ETH>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                amount = new BN(toWei(query, 'ether'))
                if (stakedEther.lt(amount)) {
                    console.log(chalk.red('Amount must not be higher than current stake.'))
                    rl.prompt()
                    break
                }

                try {
                    await node.paymentChannels.contractCall(node.paymentChannels.contract.methods.unstakeEther(amount))
                } catch (err) {
                    console.log(chalk.red(err.message))
                } finally {
                    setTimeout(() => {
                        rl.prompt()
                    })
                }
                break
            case 'openChannels':
                let str = `${chalk.yellow('ChannelId:'.padEnd(64, ' '))} - ${chalk.blue('PeerId:')}\n`

                try {
                    str += await node.paymentChannels.getAllChannels(
                        channel =>
                            pubKeyToPeerId(channel.state.restoreTransaction.counterparty).then(
                                peerId => `${chalk.yellow(channel.channelId.toString('hex'))} - ${chalk.blue(peerId.toB58String())}`
                            ),
                        promises => {
                            if (promises.length == 0) return `\n  No open channels.`

                            return Promise.all(promises).then(results => results.join('\n'))
                        }
                    )
                } catch (err) {
                    return console.log(chalk.red(err.message))
                }

                console.log(str)

                setTimeout(() => {
                    rl.prompt()
                })
                break
            case 'open':
                if (!query) {
                    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                try {
                    peerId = await checkPeerIdInput(query)
                } catch (err) {
                    console.log(err.message)
                    setTimeout(() => {
                        rl.prompt()
                    })
                    break
                }

                let interval
                node.paymentChannels
                    .open(peerId)
                    .then(() => {
                        const channelId = getId(
                            /* prettier-ignore */
                            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                            pubKeyToEthereumAddress(peerId.pubKey.marshal())
                            )

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
                if (!query) {
                    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                try {
                    peerId = await checkPeerIdInput(query)
                } catch (err) {
                    console.log(err.message)
                    setTimeout(() => {
                        rl.prompt()
                    })
                    break
                }

                rl.question(`Sending message to ${chalk.blue(peerId.toB58String())}\nType in your message and press ENTER to send:\n`, message =>
                    node
                        .sendMessage(rlp.encode([message, Date.now().toString()]), peerId)
                        .catch(err => console.log(chalk.red(err.message)))
                        .finally(() => rl.prompt())
                )
                break
            case 'closeAll':
                try {
                    await node.paymentChannels.closeChannels()
                    console.log(`${chalk.green(`Closed all channels and received`)} ${chalk.magenta(fromWei(receivedMoney.toString(), 'ether'))} ETH.`)
                } catch (err) {
                    console.log(chalk.red(err.message))
                } finally {
                    setTimeout(() => {
                        rl.prompt()
                    })
                }
                break
            case 'close':
                if (!query) {
                    console.log(chalk.red(`Invalid arguments. Expected 'close <peerId>'. Received '${input}'`))
                    rl.prompt()
                    break
                }

                try {
                    const peerInfo = node.peerBook.get(PeerId.createFromB58String(query))
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

/* prettier-ignore */
;(async function main() {
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
        return runAsBootstrapNode()
    } else {
        return runAsRegularNode()
    }
})()
