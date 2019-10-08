'use strict'

const { createHash } = require('crypto')
const { createNode } = require('../../src')
const { privKeyToPeerId, pubKeyToEthereumAddress, sendTransaction, log, createDirectoryIfNotExists, signTransaction } = require('../../src/utils')
const { STAKE_GAS_AMOUNT } = require('../../src/constants')
const { toWei, fromWei } = require('web3-utils')
const Ganache = require('ganache-core')
const BN = require('bn.js')
const LevelDown = require('leveldown')
const chalk = require('chalk')
const axios = require('axios')
const fsPromise = require('fs').promises
const querystring = require('querystring')

const MINIMAL_FUND = new BN(toWei('1.0', 'ether'))
const MINIMAL_STAKE = new BN(toWei('0.09', 'ether'))

/**
 * Allow nodes to find each other by establishing connections
 * between adjacent nodes.
 *
 * Connection from A -> B, B -> C, C -> D, ...
 *
 * @async
 * @param {Hopr} nodes nodes that will have open connections afterwards
 */
module.exports.warmUpNodes = async nodes => {
    const promises = []

    nodes.forEach((node, n) =>
        promises.push(
            new Promise((resolve, reject) => {
                node.dial(nodes[(n + 1) % nodes.length].peerInfo, (err, conn) => {
                    if (err) reject(err)

                    resolve()
                })
            })
        )
    )

    return Promise.all(promises)
}

/**
 * Create HOPR nodes, establish a connection between them and fund their corresponding
 * Ethereum account with some ether. And finally stake a fraction of that ether in order
 * open payment channel inside the HOPR contract.
 *
 * @param {number} amountOfNodes number of nodes that should be generated
 * @param {object} options
 * @param {PeerId} peerId a peerId that contains public key and private key
 * @param {number} nonce the current nonce
 * @param {function} cb the function that will be called afterwards with `(err, nodes)`
 */
module.exports.createFundedNodes = async (amountOfNodes, options, peerId, nonce) => {
    const promises = []

    for (let n = 0; n < amountOfNodes; n++) {
        promises.push(
            createNode(
                Object.assign(
                    {
                        id: n
                    },
                    options,
                    options.bootstrapServers
                )
            )
        )
    }

    const nodes = await Promise.all(promises)

    await this.warmUpNodes(nodes)

    const fundBatch = []

    nodes.forEach((node, n) => fundBatch.push(this.fundNode(node, peerId, nonce + n)))

    await Promise.all(fundBatch)

    await Promise.all(nodes.map(node => this.stakeEther(node)))

    return nodes
}

module.exports.stakeEther = async node => {
    const stakedEther = await node.paymentChannels.contract.methods
        .states(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()))
        .call({
            from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
        })
        .then(result => new BN(result.stakedEther))

    if (stakedEther.lt(MINIMAL_STAKE)) {
        return sendTransaction(
            {
                to: process.env['CONTRACT_ADDRESS'],
                value: MINIMAL_STAKE.sub(stakedEther).toString(),
                gas: STAKE_GAS_AMOUNT,
                gasPrice: process.env['GAS_PRICE'],
                nonce: node.paymentChannels.nonce
            },
            node.peerInfo.id,
            node.paymentChannels.web3
        ).then(() => {
            node.paymentChannels.nonce = node.paymentChannels.nonce + 1
            log(
                node.peerInfo.id,
                `Funded contract ${chalk.green(process.env['CONTRACT_ADDRESS'])} with ${chalk.magenta(fromWei(MINIMAL_STAKE.sub(stakedEther), 'ether'))} ETH.`
            )
        })
    }
}

/**
 * Starts a local ganache testnet.
 *
 * @returns {Promise} a promise that resolves once the ganache instance has been started,
 * otherwise it rejects.
 */
module.exports.startBlockchain = () =>
    new Promise(async (resolve, reject) => {
        createDirectoryIfNotExists('db/testnet')
        const server = Ganache.server({
            accounts: [
                {
                    balance: `0x${toWei(new BN(100), 'ether').toString('hex')}`,
                    secretKey: process.env['FUND_ACCOUNT_PRIVATE_KEY']
                }
            ],
            network_id: '1',
            gasPrice: process.env['GAS_PRICE'],
            db: LevelDown(`${process.cwd()}/db/testnet`),
            ws: true
        })
        server.listen(process.env['GANACHE_PORT'], process.env['GANACHE_HOSTNAME'], err => {
            if (err) return reject(err)

            const addr = server.address()
            console.log(
                `Successfully started local Ganache instance at 'ws://${addr.family === 'IPv6' ? '[' : ''}${addr.address}${addr.family === 'IPv6' ? ']' : ''}:${
                    addr.port
                }'.`
            )

            resolve(server)
        })
    })

/**
 * Starts a given amount of bootstrap servers.
 *
 * @param {number} amountOfNodes how much bootstrap nodes
 */
module.exports.startBootstrapServers = async amountOfNodes => {
    const promises = []

    for (let i = 0; i < amountOfNodes; i++) {
        const peerId = await privKeyToPeerId(
            createHash('sha256')
                .update(i.toString())
                .digest()
        )

        promises.push(
            createNode({ peerId, 'bootstrap-node': true }).then(node => {
                node.on('peer:connect', peer => {
                    console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
                })
                return node
            })
        )
    }

    return Promise.all(promises)
}

module.exports.wait = miliSeconds => {
    return new Promise(resolve => setTimeout(resolve, miliSeconds))
}

module.exports.openChannelFor = (fundingNode, contract, web3, nonce, partyA, partyB) => {
    return transactionHelper(
        {
            to: contract.options.address,
            gas: 190000,
            gasPrice: process.env['GAS_PRICE'],
            value: toWei('0.2', 'ether'),
            nonce: `0x${new BN(nonce).toBuffer('be').toString('hex')}`,
            data: contract.methods.createFor(partyA, partyB).encodeABI()
        },
        fundingNode,
        web3
    )
}

module.exports.stakeFor = async (fundingNode, contract, web3, nonce, beneficiary, amount = toWei('0.2', 'ether')) => {
    return transactionHelper(
        {
            to: contract.options.address,
            gas: 190000,
            gasPrice: process.env['GAS_PRICE'],
            value: amount,
            nonce: `0x${new BN(nonce).toBuffer('be').toString('hex')}`,
            data: contract.methods.stakeFor(beneficiary).encodeABI()
        },
        fundingNode,
        web3
    )
}

module.exports.fundNode2 = (fundingNode, web3, nonce, beneficiary) => {
    // const currentBalance = new BN(await web3.eth.getBalance(beneficiary))
    return transactionHelper(
        {
            to: beneficiary,
            gas: 190000,
            gasPrice: process.env['GAS_PRICE'],
            value: toWei('0.2', 'ether'),
            nonce: `0x${new BN(nonce).toBuffer('be').toString('hex')}`
        },
        fundingNode,
        web3
    )
}

module.exports.fundNode = async (node, fundingNode, nonce) => {
    const currentBalance = new BN(await node.paymentChannels.web3.eth.getBalance(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())))

    if (!nonce) nonce = await node.paymentChannels.web3.eth.getTransactionCount(pubKeyToEthereumAddress(fundingNode.pubKey.marshal()))

    if (currentBalance.lt(MINIMAL_FUND)) {
        return sendTransaction(
            {
                from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal()),
                to: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                value: MINIMAL_FUND.sub(currentBalance).toString(),
                gas: STAKE_GAS_AMOUNT,
                gasPrice: process.env['GAS_PRICE'],
                nonce: nonce
            },
            fundingNode,
            node.paymentChannels.web3
        ).then(() => {
            log(
                node.peerInfo.id,
                `Received ${chalk.magenta(fromWei(MINIMAL_FUND.sub(currentBalance), 'ether'))} ETH from ${chalk.green(
                    pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
                )}.`
            )
        })
    }
}

module.exports.verifyContractCode = async (contractPath, contractAddress) => {
    // used to remove `import 'xyz'` statements from Solidity src code
    const IMPORT_SOLIDITY_REGEX = /^\s*import(\s+).*$/gm

    const srcFileNames = await fsPromise.readdir(contractPath)

    const distinctPaths = new Set()

    const srcFilePaths = await Promise.all(
        srcFileNames.map(source => {
            const compilerOutput = require(`${contractPath}/${source}`)
            compilerOutput.metadata = JSON.parse(compilerOutput.metadata)

            return Promise.all(
                Object.keys(compilerOutput.metadata.sources).map(async srcPath => {
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

    srcFilePaths.flat().forEach(path => distinctPaths.add(path))

    const promises = []

    distinctPaths.forEach(path => promises.push(fsPromise.readFile(path)))

    const concatenatedSourceCode = (await Promise.all(promises)).map(source => source.toString().replace(IMPORT_SOLIDITY_REGEX, '')).join('\n')

    const compilerMetadata = require(`${process.cwd()}/build/contracts/HoprChannel.json`).metadata

    let apiSubdomain = 'api'
    switch (process.env['NETWORK'].toLowerCase()) {
        case 'ropsten':
            apiSubdomain += '-ropsten'
            break
        case 'rinkeby':
            apiSubdomain += '-rinkeby'
            break
        default:
    }

    return axios
        .post(
            `https://${apiSubdomain}.etherscan.io/api`,
            querystring.stringify({
                apikey: process.env['ETHERSCAN_API_KEY'],
                module: 'contract',
                action: 'verifysourcecode',
                contractaddress: contractAddress,
                sourceCode: concatenatedSourceCode,
                contractname: 'HoprChannel',
                compilerVersion: `v${compilerMetadata.compiler.version}`,
                optimizationUsed: compilerMetadata.settings.optimizer.enabled ? '1' : '0',
                runs: compilerMetadata.settings.optimizer.runs.toString(),
                licenseType: '1'
            })
        )
        .then(response => {
            if (response.statusText !== 'OK' || !response.data) {
                console.log(`Failed to verify contract due to '${statusText}'.`)
                console.log(`Got this response: `, response)
                return
            }

            switch (parseInt(response.data.status)) {
                case 1:
                    console.log(`Successfully verified contract ${chalk.green(contractAddress)} on ${chalk.magenta(process.env['NETWORK'].toLowerCase())}`)
                    break
                case 0:
                    console.log(`Failed to verify contract due to '${response.data.result}'.`)
                    break
                default:
                    console.log(`${response.data.message} ${response.data.result}`)
            }
        })
        .catch(error => {
            console.log(error.message)
        })
}

async function transactionHelper(tx, fundingNode, web3) {
    const signedTransaction = await signTransaction(tx, fundingNode, web3)

    return web3.eth.sendSignedTransaction.request(signedTransaction.rawTransaction)
}
