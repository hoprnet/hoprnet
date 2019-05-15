'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

var myEnv = dotenv.config()
dotenvExpand(myEnv)

const readline = require('readline')
const chalk = require('chalk')
const { waterfall, forever } = require('neo-async')
const Hopr = require('./src')
const read = require('read')
const getopts = require('getopts')
const { getId, pubKeyToEthereumAddress, pubKeyToPeerId, sendTransaction, bufferToNumber } = require('./src/utils')
const { STAKE_GAS_AMOUNT } = require('./src/constants')
const PeerId = require('peer-id')
const BN = require('bn.js')
const { toWei, fromWei } = require('web3-utils')
const rlp = require('rlp')
const secp256k1 = require('secp256k1')
const bs58 = require('bs58')
const Transaction = require('./src/transaction')
const pullAll = require('lodash.pullall')

const MINIMAL_FUNDS = new BN(toWei('0.15', 'ether'))
const MINIMAL_STAKE = new BN(toWei('0.10', 'ether'))
const DEFAULT_STAKE = new BN(toWei('0.11', 'ether'))

let node, funds, ownAddress, stakedEther, rl

function isBootstrapNode(peerId) {
    const channelId = getId(
        node.peerInfo.id.pubKey.marshal(),
        peerId.pubKey.marshal()
    )

    const isBootstrapNode = node.bootstrapServers
        .some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerId))

    return !isBootstrapNode
}

function getExistingChannels(callback) {
    let existingChannels = []

    node.db.createValueStream({
        gt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 0)),
        lt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 255))
    })
        .on('data', (value) => {
            const restoreTx = Transaction.fromBuffer(value)
            const peerId = pubKeyToPeerId(secp256k1.recover(restoreTx.hash, restoreTx.signature, bufferToNumber(restoreTx.recovery)))

            existingChannels.push(peerId)
        })
        .on('end', () =>
            callback(null, existingChannels.filter(isBootstrapNode).map((peerId) => peerId.toB58String()))
        )
        .on('error', (err) => cb(err))
}

const keywords = ['open', 'stake', 'stakedEther', 'unstake', 'send', 'quit', 'crawl', 'openChannels', 'closeAll']
function tabCompletion(line, cb) {
    const operands = line.trim().split(' ')

    let hits
    switch (operands[0].toLowerCase()) {
        case 'send':
            hits = node.peerBook.getAllArray()
                .filter((peerInfo) => {
                    if (operands.length > 1 && !peerInfo.startsWith(operands[1]))
                        return false

                    if (node.bootstrapServers
                        .some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id)))
                        return false

                    return true
                })

            if (!hits.length) {
                console.log(chalk.red(`\nDoesn't know any other node except the bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}!`))
                return cb(null, [[''], line])
            }

            return cb(null, [hits.map((peerInfo) => `send ${peerInfo.id.toB58String()}`), line])
        case 'stake':
            if (funds.isZero()) {
                console.log(chalk.red(`\nCan't stake any funds without any Ether.`))
                return [['stake 0.00'], line]
            }

            return cb(null, [[`stake ${fromWei(funds)}`], line])

        case 'unstake':
            return cb(null, [[`unstake ${fromWei(stakedEther, 'ether')}`], line])
        case 'open':
            getExistingChannels((err, existingChannels) => {
                if (err) {
                    console.log(err.message)
                    return cb(null, [[''], line])
                }

                const peers = node.peerBook.getAllArray().filter((peerInfo) => isBootstrapNode(peerInfo.id)).map((peerInfo) => peerInfo.id.toB58String())

                pullAll(peers, existingChannels)

                if (peers.length < 1) {
                    console.log(chalk.red(`\nDoesn't know any node to open a payment channel with.`))
                    return cb(null, [[''], line])
                }

                hits = peers.filter((peerId) => peerId.startsWith(operands[1]))

                if (hits.length == 0)
                    hits = peers

                return cb(null, [hits.map((str) => `open ${str}`), line])
            })
            break
        case 'close':
            getExistingChannels((err, existingChannels) => {
                if (err) {
                    console.log(err.message)
                    return cb(null, [[''], line])
                }

                if (existingChannels.length < 1) {
                    console.log(chalk.red(`\nCan't close a payment channel as there aren't any open ones!`))
                    return cb(null, [[''], line])
                }

                hits = existingChannels.filter((peerId) => peerId.startsWith(operands[1]))

                if (hits.length == 0)
                    hits = existingChannels

                return cb(null, [hits.map((str) => `close ${str}`), line])
            })
            break
        default:
            hits = keywords.filter((keyword) => keyword.startsWith(line))

            return cb(null, [hits.length ? hits : keywords, line])
    }
}

function connectToBootstrapNode() {
    return Promise.all(
        node.bootstrapServers.map((addr) => new Promise((resolve, reject) =>
            node.dial(addr, (err, conn) => {
                if (err)
                    return resolve(false)

                resolve(true)
            })
        ))
    ).then((values) => {
        if (!values.reduce((acc, value) => acc || value))
            throw Error('Unable to connect to any bootstrap server.')
    })
}

function stakeEther(amount) {
    if (!amount)
        amount = DEFAULT_STAKE

    return sendTransaction({
        from: ownAddress,
        to: process.env.CONTRACT_ADDRESS,
        value: amount,
        gas: STAKE_GAS_AMOUNT
    }, node.peerInfo.id, node.paymentChannels.web3).then(() => {
        node.paymentChannels.nonce = node.paymentChannels.nonce + 1
    })
}

function stopNode() {
    node.stop((err) => {
        if (err)
            process.exit(1)

        process.exit(0)
    })
}


async function main() {
    const options = getopts(process.argv.slice(2), {
        alias: {
            b: "bootstrap-node",
            m: "send-messages",
        }
    })

    console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

    if (options['bootstrap-node'])
        console.log(`... running as bootstrap node!.`)

    if (Array.isArray(options._) && options._.length > 0) {
        options.id = Number.parseInt(options._[0])
    }
    try {
        node = await Hopr.createNode(options)
    } catch (err) {
        console.log(chalk.red(err.message))
        process.exit(1)
    }

    ownAddress = pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())

    console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)

    if (options['bootstrap-node']) {
        node.on('peer:connect', (peer) => {
            console.log(`Incoming connection from ${peer.id.toB58String()}.`)
        })
    }

    if (!options['bootstrap-node']) {
        try {
            let results = await Promise.all([
                node.paymentChannels.web3.eth.getBalance(ownAddress),
                node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress })
            ])
            funds = new BN(results[0])
            stakedEther = new BN(results[1].stakedEther)
        } catch (err) {
            console.log(err)
            return
        }

        console.log(
            `Own Ethereum address:\n` +
            `\t${pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())}\n` +
            `\tPrivate key: ${node.peerInfo.id.privKey.marshal().toString('hex')}\n` +
            `\tFunds: ${fromWei(funds, 'ether')} ETH\n` +
            `\tStake: ${fromWei(stakedEther, 'ether')} ETH`
        )

        if (funds.lt(MINIMAL_FUNDS))
            throw Error(`Insufficient funds. Got only ${fromWei(funds.toString(), 'ether')} ETH. Please fund the account with at least ${fromWei((MINIMAL_FUNDS).sub(funds), 'Ether')} ETH.`)

        if (stakedEther.lt(new BN(toWei('0.1', 'ether'))))
            console.log(chalk.red(`Staked ether is less than ${MINIMAL_STAKE} ETH.`))

        console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

        try {
            await connectToBootstrapNode(node)
        } catch (err) {
            console.log(chalk.red(err.message))
            stopNode()
        }

        rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
            completer: tabCompletion
        })

        // Scan restore transactions and detect whether they're on-chain available

        rl.on('line', (input) => {
            rl.pause()
            const operands = input.trim().split(' ')
            let amount
            switch (operands[0]) {
                case 'crawl':
                    node.crawlNetwork((err) => {
                        if (err)
                            console.log(chalk.red(err.message))

                        rl.prompt()
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
                    node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress })
                        .then((state) => {
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

                    node.paymentChannels.contractCall(node.paymentChannels.contract.methods.unstakeEther(amount.toString()))
                        .then(() => {
                            rl.prompt()
                        })
                        .catch((err) => {
                            console.log(err)
                            rl.prompt()
                        })
                    break
                case 'openChannels':
                    let str = `${chalk.yellow('ChannelId:'.padEnd(64, ' '))} - ${chalk.blue('PeerId:')}`
                    let index = 0
                    node.db.createReadStream({
                        gt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 0)),
                        lt: node.paymentChannels.RestoreTransaction(Buffer.alloc(32, 255))
                    }).on('data', ({ key, value }) => {
                        index++
                        const channelId = key.slice(key.length - 32)
                        const tx = Transaction.fromBuffer(value)
                        const peerId = pubKeyToPeerId(secp256k1.recover(tx.hash, tx.signature, bufferToNumber(tx.recovery)))
                        str += `\n${chalk.yellow(channelId.toString('hex'))} - ${chalk.blue(peerId.toB58String())}`
                    }).on('end', () => {
                        if (index == 0)
                            str += `\n  No open channels.`

                        console.log(str)
                        rl.prompt()
                    })
                    break
                case 'open':
                    if (operands.length != 2) {
                        console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${input}'`))
                        rl.prompt()
                        break
                    }

                    try {
                        const peerInfo = node.peerBook.get(new PeerId(bs58.decode(operands[1])))
                        const channelId = getId(
                            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                            pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal())
                        )
                        let interval
                        node.paymentChannels.registerOpeningListener(channelId, () => {
                            console.log(chalk.green(`Successfully opened channel ${channelId.toString('hex')}.`))
                            clearInterval(interval)
                            rl.prompt()
                        })
                        node.paymentChannels.open(peerInfo.id)
                        rl.write(`Submitted transaction. Waiting for confirmation .`)
                        interval = setInterval(() => rl.write('.'), 1000)

                    } catch (err) {
                        console.log(chalk.red('Unable to open payment channel.'))
                        rl.prompt()
                    }
                    break
                case 'send':

                case 'closeAll':
                    node.paymentChannels.closeChannels((err) => {
                        if (err)
                            console.log(chalk.red(err.message))

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
                        const peerInfo = node.peerBook.get(new PeerId(bs58.decode(operands[1])))
                        const channelId = getId(
                            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                            pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal())
                        )
                        let interval
                        node.paymentChannels.onceClosed(channelId, (err, receivedMoney) => {
                            console.log(`${chalk.green(`Successfully closed channel`)} ${chalk.yellow(channelId.toString('hex'))}`)
                            clearInterval(interval)
                            node.paymentChannels.deleteChannel(channelId).then(() =>
                                setTimeout(() => {
                                    readline.clearLine(process.stdin, 0)
                                    rl.prompt()
                                })
                            )
                        })

                        function close() {
                            node.paymentChannels.requestClose(channelId, true)
                            rl.write(`Submitted transaction. Waiting for confirmation .`)
                            interval = setInterval(() => rl.write('.'), 1000)
                        }

                        node.db.get(node.paymentChannels.Transaction(channelId), (err, tx) => {
                            if (err && !err.notFound) {
                                console.log(chalk.red(err.message))
                                rl.prompt()
                            } else if (err && err.notFound) {
                                rl.question(
                                    `Haven't found any transaction on that channel.\n` +
                                    `Do you want to use the restore transaction to close the channel? (${chalk.red('Y')}/${chalk.green('n')}): `,
                                    (answer) => {
                                        switch (answer.toLowerCase()) {
                                            case '':
                                            case 'y':
                                                close()
                                                break
                                            case 'n':
                                                rl.prompt()
                                                break
                                            default:
                                                console.log(chalk.red('Invalid argument.'))
                                                rl.prompt()
                                                break
                                        }
                                    })
                            } else {
                                close()
                            }
                        })
                    } catch (err) {
                        console.log(err.message)
                        console.log(chalk.red('Unable to close payment channel.'))
                        rl.prompt()
                    }
                    break

                default:
                    console.log(chalk.red('Unknown command!'))
                    rl.prompt()
            }
        })
        rl.prompt()

        // if (options['send-messages']) {
        //     const sendMessage = () => {
        //         const recipient = randomSubset(node.peerBook.getAllArray(), 1, (peerInfo) =>
        //             !options.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
        //         )
        //         return node.sendMessage('Psst ... secret message from Validity Labs!@' + Date.now().toString(), recipient[0].id, () => { })
        //     }
        //     setInterval(sendMessage, 90 * 1000)
        // } else if (options['bootstrap-node']) {
        //     return
        // } else {
        //     return sendMessages(node, () => { })
        // }
    }
}

main()

function sendMessages(node, cb) {
    forever((cb) => waterfall([
        (cb) => selectRecipient(node, cb),
        (destination, cb) => {
            console.log('Type in your message')
            read({
                edit: true
            }, (err, message) => {
                if (err)
                    process.exit(0)

                console.log(`Sending "${message}" to \x1b[34m${destination.id.toB58String()} \x1b[0m.\n`)

                const encodedMessage = rlp.encode([message, Date.now().toString()])
                node.sendMessage(encodedMessage, destination.id, cb)

                cb()
            })
        }
    ], cb))
}