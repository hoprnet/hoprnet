require('../config')

const cluster = require('cluster')

const NUMBER_OF_NODES = 4

const Web3 = require('web3')

const { toWei, fromWei } = require('web3-utils')
const BN = require('bn.js')

const { STAKE_GAS_AMOUNT } = require('../src/constants')
const { deployContract, signTransaction, privKeyToPeerId, pubKeyToEthereumAddress, log } = require('../src/utils')

const { createNode } = require('../src')
const { startBlockchain, wait } = require('./utils')

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const rlp = require('rlp')

const MINIMAL_FUND = new BN(toWei('0.11', 'ether'))

;(async () => {
    if (cluster.isMaster) {
        console.log(`Master ${process.pid} is running`)

        // start local testnet

        await startChildProcess({
            testnet: true
        })

        const web3 = new Web3(process.env['PROVIDER'])

        let nonce = await web3.eth.getTransactionCount(process.env['FUND_ACCOUNT_ETH_ADDRESS'])

        const contractAddress = await deployContract(nonce, web3)

        const abi = require('../build/contracts/HoprChannel.json').abi

        const fundingNode = await privKeyToPeerId(process.env['FUND_ACCOUNT_PRIVATE_KEY'])

        const contract = new web3.eth.Contract(abi, contractAddress, {
            from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
        })

        const batch = new web3.eth.BatchRequest()
        for (let i = 0; i < NUMBER_OF_NODES; i++) {
            let peerId = await privKeyToPeerId(process.env[`DEMO_ACCOUNT_${i}_PRIVATE_KEY`])

            batch.add(
                contract.methods.stakeFor(pubKeyToEthereumAddress(peerId.pubKey.marshal())).send.request({
                    from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal()),
                    value: MINIMAL_FUND.toString(),
                    nonce: ++nonce
                })
            )

            const tx = await signTransaction(
                {
                    from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal()),
                    to: pubKeyToEthereumAddress(peerId.pubKey.marshal()),
                    gas: STAKE_GAS_AMOUNT,
                    gasPrice: process.env.GAS_PRICE,
                    value: MINIMAL_FUND,
                    nonce: ++nonce
                },
                fundingNode,
                web3
            )

            batch.add(web3.eth.sendSignedTransaction.request(tx.rawTransaction))
        }

        await batch.execute()

        console.log('funding done')

        await startChildProcess({
            'bootstrap-node': true
        })

        // start HOPR nodes
        const promises = []
        for (let i = 0; i < NUMBER_OF_NODES; i++) {
            promises.push(
                startChildProcess({
                    id: i
                })
            )
        }

        const childs = await Promise.all(promises)

        // Tell all nodes that they can start sending messages
        childs.forEach(child => child.send('go'))

        // cluster.on('exit', (worker, code, signal) => {
        //     console.log(`worker ${worker.process.pid} died`)
        // })
    } else {
        if (process.env['testnet']) {
            console.log(`Worker ${process.pid} started as testnet`)
            return startAsTestnet()
        }

        if (process.env['bootstrap-node']) {
            console.log(`Worker ${process.pid} started as bootstrap node`)
            return startAsBootstrapNode()
        }

        const peerInfo = new PeerInfo(PeerId.createFromB58String(`16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf`))

        peerInfo.multiaddrs.add(`/ip4/127.0.0.1/tcp/${process.env['PORT']}/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf`)

        const node = await createNode({
            id: parseInt(process.env['id']),
            bootstrapServers: [peerInfo]
        })

        await waitForStartMessage(node)

        try {
            await node.crawler.crawl(peerInfo => isNotBootstrapNode(node, peerInfo.id))
        } catch (err) {
            console.log(err)
        }

        await sendMessageAndCloseChannels(node, 0, 3)

        await wait(3 * 1000)

        await sendMessageAndCloseChannels(node, 3, 0)

        await wait(3 * 1000)

        await sendMessageAndCloseChannels(node, 2, 1)

    }
})()

function startAsTestnet() {
    return startBlockchain().then(() => process.send('successfully started'))
}

function startAsBootstrapNode() {
    return createNode({
        'bootstrap-node': true
    }).then(() => process.send('successfully started'))
}

function startChildProcess(forkEnvironment) {
    return new Promise((resolve, reject) => {
        const cl = cluster.fork(forkEnvironment)

        cl.once('message', msg => {
            switch (msg) {
                case 'successfully started':
                    return resolve(cl)
                default:
                    return reject(Error(`Invalid message. Got '${msg}'.`))
            }
        })
    })
}

function waitForStartMessage() {
    return new Promise((resolve, reject) => {
        process.once('message', msg => {
            switch (msg) {
                case 'go':
                    return resolve()
                default:
                    return reject(Error(`invalid msg. Got '${msg}'.`))
            }
        })

        process.send('successfully started')
    })
}

async function sendMessageAndCloseChannels(node, sender, receiver) {
    if (parseInt(process.env['id']) == sender) {
        await node.sendMessage(
            rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]),
            await privKeyToPeerId(process.env[`DEMO_ACCOUNT_${receiver}_PRIVATE_KEY`])
        )
    }

    await wait(3 * 1000)

    const receivedMoney = await node.paymentChannels.closeChannels()

    log(node.peerInfo.id, `Closed all channels and received ${fromWei(receivedMoney, 'ether').toString()}.`)
}

function isNotBootstrapNode(node, peerId) {
    return !node.bootstrapServers.some(peerInfo => peerInfo.id.isEqual(peerId))
}
