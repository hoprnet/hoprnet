require('../config')

const rlp = require('rlp')

const Web3 = require('web3')

const { createNode } = require('../src')

const { pubKeyToEthereumAddress, privKeyToPeerId, deployContract } = require('../src/utils')
const { startBootstrapServers, startBlockchain, wait, openChannelFor, stakeFor, fundNode2 } = require('./utils')

const AMOUNT_OF_NODES = 4

;(async () => {
    const bootstrapServers = await startBootstrapServers(1)

    const web3 = new Web3(process.env.PROVIDER)

    await startBlockchain()

    const fundingNode = await privKeyToPeerId(process.env['FUND_ACCOUNT_PRIVATE_KEY'])

    let nonce = await web3.eth.getTransactionCount(pubKeyToEthereumAddress(fundingNode.pubKey.marshal()))

    const contractAddress = await deployContract(nonce++, web3)

    const abi = require('../build/contracts/HoprChannel.json').abi
    const contract = new web3.eth.Contract(abi, contractAddress, {
        from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
    })

    const promises = []
    for (let i = 0; i < AMOUNT_OF_NODES; i++) {
        promises.push(
            createNode({
                id: i,
                contractAddress,
                bootstrapServers: bootstrapServers.map(node => node.peerInfo)
            })
        )
    }

    const nodes = await Promise.all(promises)

    const batch = new web3.eth.BatchRequest()

    let partyA, partyB
    for (let i = 0; i < AMOUNT_OF_NODES; i++) {
        partyA = pubKeyToEthereumAddress(nodes[i].peerInfo.id.pubKey.marshal())

        batch.add(await stakeFor(fundingNode, contract, web3, nonce++, partyA))
        batch.add(await fundNode2(fundingNode, web3, nonce++, partyA))
        for (let j = i + 1; j < AMOUNT_OF_NODES - 1; j++) {
            partyB = pubKeyToEthereumAddress(nodes[j].peerInfo.id.pubKey.marshal())

            batch.add(await openChannelFor(fundingNode, contract, web3, nonce++, partyA, partyB))
        }
    }

    await batch.execute()

    await wait(2 * 1000)

    await nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[2].peerInfo.id)

    await wait(2 * 1000)

    await nodes[2].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[1].peerInfo.id)

    await wait(2 * 1000)

    await nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[1].peerInfo.id)

    await wait(2 * 1000)

    await Promise.all(nodes.map(node => node.paymentChannels.closeChannels()))
})()