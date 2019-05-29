'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

const myEnv = dotenv.config()
dotenvExpand(myEnv)

const readline = require('readline')
const chalk = require('chalk')
const Hopr = require('./src')
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

const MINIMAL_STAKE = new BN(toWei('0.10', 'ether'))
const DEFAULT_STAKE = new BN(toWei('0.11', 'ether'))

let node, funds, ownAddress, stakedEther, rl, options

function parseOptions() {
    const options = require('getopts')(process.argv.slice(2), {
        alias: {
            b: "bootstrap-node",
            m: "send-messages",
        }
    })

    if (Array.isArray(options._) && options._.length > 0) {
        options.id = Number.parseInt(options._[0])
    }

    return options
}

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
                    if (operands.length > 1 && !peerInfo.id.toB58String().startsWith(operands[1]))
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

                hits = operands.length > 1 ? peers.filter((peerId) => peerId.startsWith(operands[1])) : peers

                return cb(null, [hits.length ? hits.map((str) => `open ${str}`) : ['open'], line])
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

                hits = operands.length > 1 ? existingChannels.filter((peerId) => peerId.startsWith(operands[1])) : existingChannels

                return cb(null, [hits.length ? hits.map((str) => `close ${str}`) : ['close'], line])
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

function runAsBootstrapNode() {
    if (options['bootstrap-node'])
        console.log(`... running as bootstrap node!.`)

    node.on('peer:connect', (peer) => {
        console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
    })
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

    if (options['bootstrap-node'])
        return runAsBootstrapNode()

    ownAddress = pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())

    try {
        [funds, stakedEther] = await Promise.all([
            node.paymentChannels.web3.eth.getBalance(ownAddress).then((result) => new BN(result)),
            node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress }).then((result) => new BN(result.stakedEther))
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
        console.log(chalk.red(`Insufficient funds. Got only ${fromWei(stakedEther.add(funds), 'ether')} ETH to stake. Please fund the account ${chalk.green(ownAddress.toString())} with at least ${fromWei((MINIMAL_STAKE).sub(funds).sub(stakedEther), 'Ether')} ETH.`))
        return stopNode()
    }

    if (stakedEther.lt(MINIMAL_STAKE)) {
        await new Promise((resolve, reject) =>
            rl.question(`Staked Ether is less than ${fromWei(MINIMAL_STAKE, 'ether')} ETH. Do you want to refund now? (${chalk.green('Y')}/${chalk.red('n')}): `, (answer) => {
                switch (answer.toLowerCase()) {
                    case '':
                    case 'y':
                        rl.question(`Amount? : `, (answer) => {
                            resolve(stakeEther(toWei(answer)))
                        })
                        rl.write(fromWei(MINIMAL_STAKE.sub(stakedEther), 'ether'))
                        break
                    default:
                        return stopNode()
                }
            })
        )
    }

    console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

    try {
        await connectToBootstrapNode(node)
    } catch (err) {
        console.log(chalk.red(err.message))
        return stopNode()
    }

    // Scan restore transactions and detect whether they're on-chain available

    rl.on('line', (input) => {
        rl.pause()
        const operands = input.trim().split(' ')
        let amount
        switch (operands[0]) {
            case 'crawl':
                node.crawlNetwork((peerInfo) => !node.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))                )
                    .catch((err) => console.log(chalk.red(err.message)))
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

                let peerInfo
                try {
                    peerInfo = node.peerBook.get(new PeerId(bs58.decode(operands[1])))
                } catch (err) {
                    console.log(chalk.red('Unable to open payment channel.'))
                    rl.prompt()
                }

                const channelId = getId(
                    pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                    pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal())
                )

                let interval
                node.paymentChannels.open(peerInfo.id)
                    .then(() => {
                        console.log(`${chalk.green(`Successfully opened channel`)} ${chalk.yellow(channelId.toString('hex'))}`)
                    })
                    .catch((err) => {
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

                rl.question(`Sending message to ${chalk.blue(operands[1])}\nType in your message and press ENTER to send:\n`, (message) =>
                    node.sendMessage(rlp.encode([message, Date.now().toString()]), operands[1], (err) => {
                        if (err)
                            console.log(chalk.red(err.message))

                        rl.prompt()
                    })
                )
                break
            case 'closeAll':
                node.paymentChannels.closeChannels()
                    .then((receivedMoney) => {
                        console.log(`${chalk.green(`Closed all channels and received`)} ${chalk.magenta(fromWei(receivedMoney.toString(), 'ether'))} ETH.`)
                        rl.prompt()
                    })
                    .catch((err) => {
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
                    const peerInfo = node.peerBook.get(new PeerId(bs58.decode(operands[1])))
                    const channelId = getId(
                        pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                        pubKeyToEthereumAddress(peerInfo.id.pubKey.marshal())
                    )

                    let interval
                    node.paymentChannels.closeChannel(channelId)
                        .then((receivedMoney) => {
                            console.log(`${chalk.green(`Successfully closed channel`)} ${chalk.yellow(channelId.toString('hex'))}. Received ${chalk.magenta(fromWei(receivedMoney, 'ether'))} ETH.`)
                            clearInterval(interval)
                            setTimeout(() => {
                                readline.clearLine(process.stdin, 0)
                                rl.prompt()
                            })
                        })
                        .catch((err) => {
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

main()