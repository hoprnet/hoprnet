require('../config')

const cluster = require('cluster')

const NUMBER_OF_NODES = 4

const Web3 = require('web3')

const { toWei } = require('web3-utils')
const BN = require('bn.js')

const { STAKE_GAS_AMOUNT } = require('../src/constants')
const { deployContract, signTransaction, privKeyToPeerId, pubKeyToEthereumAddress } = require('../src/utils')

const rlp = require('rlp')

const MINIMAL_FUND = new BN(toWei('0.11', 'ether'))

if (cluster.isMaster) {
    console.log(`Master ${process.pid} is running`)

    // start local testnet
    cluster
        .fork({
            testnet: true
        })
        .on('message', async msg => {
            switch (msg) {
                case 'successfully started':
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

                    // start bootstrap node
                    cluster
                        .fork({
                            'bootstrap-node': true
                        })
                        .on('message', msg => {
                            switch (msg) {
                                case 'successfully started':
                                    // // start HOPR nodes
                                    for (let i = 0; i < NUMBER_OF_NODES; i++) {
                                        cluster.fork({
                                            id: i
                                        })
                                    }
                                    break
                                default:
                                    console.log(msg)
                            }
                        })
                    break
                default:
                    console.log(msg)
            }
        })

    // cluster.on('exit', (worker, code, signal) => {
    //     console.log(`worker ${worker.process.pid} died`)
    // })
} else {
    if (process.env['testnet']) {
        console.log(`Worker ${process.pid} started as testnet`)
        return require('./utils')
            .startBlockchain()
            .then(() => process.send('successfully started'))
    }

    const { createNode } = require('../src')

    if (process.env['bootstrap-node']) {
        console.log(`Worker ${process.pid} started as bootstrap node`)
        return createNode({
            'bootstrap-node': true
        }).then(() => process.send('successfully started'))
    }

    const PeerId = require('peer-id')
    const PeerInfo = require('peer-info')
    const { privKeyToPeerId } = require('../src/utils')

    const peerInfo = new PeerInfo(PeerId.createFromB58String(`16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf`))

    peerInfo.multiaddrs.add(`/ip4/127.0.0.1/udp/${process.env['PORT']}/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf`)
    ;(async () => {
        const node = await createNode({
            id: parseInt(process.env['id']),
            bootstrapServers: [peerInfo]
        })

        function isNotBootstrapNode(peerId) {
            return !node.bootstrapServers.some(peerInfo => peerInfo.id.isEqual(peerId))
        }

        try {
            await node.crawler.crawl(peerInfo => isNotBootstrapNode(peerInfo.id))
        } catch (err) {
            console.log(err)
        }

        if (parseInt(process.env['id']) == 0) {
            await node.sendMessage(
                rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]),
                await privKeyToPeerId(process.env[`DEMO_ACCOUNT_3_PRIVATE_KEY`])
            )
        }
    })()
}
