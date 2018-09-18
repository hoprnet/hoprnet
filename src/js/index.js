'use strict'

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const WS = require('libp2p-websockets')
const Mplex = require('libp2p-mplex')
const SPDY = require('libp2p-spdy')
const SECIO = require('libp2p-secio')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const KadDHT = require('libp2p-kad-dht')
const defaultsDeep = require('@nodeutils/defaults-deep')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')

const secp256k1 = require('secp256k1')
const multihash = require('multihashes')

const Record = require('../record')
const constants = require('./constants')
const HeaderTest = require('./messageDelivery/test')
const Header = require('./messageDelivery/header')
const messageDeliveryHandler = require('./messageDelivery/handlers')

const prp = require('./messageDelivery/prp')
const dht = require('../dht')

// const BS = require('./dht')
// const Bootstrap = new BS(['/ip4/0.0.0.0/tcp/0'])

const pull = require('pull-stream')

const fs = require('fs')

const crypto = require('libp2p-crypto')

// PeerId.create({
//     bits: 4096
// }, (err, peerid) => {
//     fs.writeFileSync('./keys/fred.json', JSON.stringify(peerid.toJSON()))
//     console.log(JSON.stringify(peerid.toJSON()))
// })

class MyBundle extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP, WS],
                streamMuxer: [Mplex],
                // connEncryption: [SECIO],
                dht: KadDHT
            },
            config: {
                dht: {
                    kBucketSize: 20
                },
            //     EXPERIMENTAL: {
            //         dht: true
            //     }
             }
        }

        super(defaultsDeep(_options, defaults))
    }
}

function createNode(callback, keyPath, addrs) {
    let node

    waterfall([
        (cb) => fs.readFile(keyPath, cb),
        (jsonKey, cb) => {
            crypto.keys.generateKeyPair('secp256k1', 256, (err, key) => {
                let id = multihash.encode(key.public.bytes, 'sha2-256')
                PeerInfo.create(new PeerId(id, key, key.public), cb)
            })
        },
        (peerInfo, cb) => {
            addrs.forEach(addr => peerInfo.multiaddrs.add(addr))
            node = new MyBundle({
                peerInfo
            })
            node.start(cb)
        }
    ], (err) => callback(err, node))
}


parallel([
    (cb) => createNode(cb, './keys/alice.json', ['/ip4/0.0.0.0/tcp/0']),
    (cb) => createNode(cb, './keys/bob.json', ['/ip4/0.0.0.0/tcp/0'])
    // (cb) => createNode(cb, './keys/chris.json', ['/ip4/0.0.0.0/tcp/0'])
], (err, nodes) => {
    if (err) { throw err }

    let foo = HeaderTest.createTestHeader(2)
    let msg = Buffer.from(
        'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ' +
        'ut labore et dolore magna aliqua. Netus et malesuada fames ac turpis egestas integer eget ' +
        'aliquet. Commodo viverra maecenas accumsan lacus vel. Mauris a diam maecenas sed enim ut sem ' +
        'viverra. Habitasse platea dictumst vestibulum rhoncus est. Eget nullam non nisi est sit amet ' +
        'facilisis. In ante metus dictum at tempor. Auctor augue mauris augue neque gravida in. Ac ' +
        'auctor augue mauris augue neque gravida. Sit amet aliquam id diam. Sed turpis tincidunt id ' +
        'aliquet risus feugiat. Tristique nulla aliquet enim tortor at auctor urna nunc id. Nec dui ' +
        'nunc mattis enim. Congue eu consequat ac felis.')
    let ciphertext = msg

    nodes.forEach((node, index) => {
        messageDeliveryHandler(node, foo.keys[index].privKey)
        // dht.registerHandlers(node)
    })

    const node1 = nodes[0]
    const node2 = nodes[1]
    // const node3 = nodes[2]

    foo.secrets.forEach(secret => {
        let { key, iv } = Header.deriveCipherParameters(secret)
        ciphertext = prp.createPRP(key, iv).permutate(ciphertext)
    })

    // let plaintext = ciphertext
    // foo.secrets.reverse().forEach(secret => {
    //     let { key, iv } = Header.deriveCipherParameters(secret)
    //     plaintext = prp.createPRP(key, iv).inverse(plaintext)
    // })



    node2.dialProtocol(node1.peerInfo, constants.relayProtocol, (err, conn) => {
        pull(
            pull.values([foo.header.alpha, foo.header.beta, foo.header.gamma, ciphertext]),
            conn
        )
    })

    // const record = new Record(
    //     node1.peerInfo.id.privKey,
    //     node1.peerInfo.id.pubKey,
    //     null,
    //     null,
    //     null
    // )

    // dht.putRecord(node1, node2, [record.toString()], (err) => {
    //     if (err) { throw err }
    //     dht.getRecord(node1, node2, node1.peerInfo.id, (err, data) => {
    //         console.log(err, data)
    //     })
    // })



    // Bootstrap.start((err) => {
    //     if (err) { throw err }

    //     Bootstrap.askForPeer(node1, [node1.peerInfo.id, node1.peerInfo.id], (err, data) => {
    //         console.log(err, data)

    //     })
    // })




})



        // function (node) {
        //     node.handle('/nothing')
        // }


    // node1.dialProtocol(node2.peerInfo, constants.relayProtocol, (err, conn) => {
    //     pull(
    //         pull.values([node2.peerInfo.id.toBytes(), 'test']),
    //         conn
    //     )

    //     node2.dialProtocol(node3.peerInfo, constants.relayProtocol, (err, conn) => {
    //         pull(
    //             pull.values([node3.peerInfo.id.toBytes(), 'test']),
    //             conn
    //         )

    //     })
    // })


//     parallel([
//         (cb) => node1.dial(node2.peerInfo, (err) => {
//             if (err) { throw err }
//             node1.hangUp(node2.peerInfo, cb)
//         }),
//         (cb) => node2.dial(node3.peerInfo, (err) => {
//             if (err) { throw err }
//             node1.hangUp(node3.peerInfo, cb)
//         }),
//         // Set up of the cons might take time
//         (cb) => setTimeout(cb, 300)
//     ], (err) => {
//         if (err) { throw err }


//         node1.peerRouting.findPeer(node2.peerInfo.id, (err, peer) => {
//             if (err) { throw err }

//             console.log('Found it, multiaddrs are:')
//             // peer.multiaddrs.forEach((ma) => console.log(ma.toString()))

//             let key = Buffer.from('/pk/' + node1.peerInfo.id.toB58String())
//             let value = Buffer.from('/foo .....................................')

//             node1.dht.put(key, value, (err) => {
//                 if (err) { throw err }
//                 node1.dht.get(key, (err, value) => {
//                     console.log(value.toString())
//                 })
//             })

//             // node1.dialProtocol(node3.peerInfo, constants.relayProtocol, (err, conn) => {
//             //     pull(
//             //         pull.values([node1.peerInfo.id.toBytes(), 'test']),
//             //         conn
//             //     )
//             // })
//         })
//     })
// })




























// 'use strict'

// const libp2p = require('libp2p')
// const TCP = require('libp2p-tcp')
// const WS = require('libp2p-websockets')
// const SECIO = require('libp2p-secio')
// const MPLEX = require('libp2p-mplex')
// const libp2pKadDHT = require('libp2p-kad-dht')
// const PeerInfo = require('peer-info')
// const defaultsDeep = require('@nodeutils/defaults-deep')
// const crypto = require('crypto')

// const registerRelayFunctionality = require('./relay')
// const constants = require('./constants')
// const bootstrap = require('./bootstrap')


// const waterfall = require('async/waterfall')
// const parallel = require('async/parallel')
// const applyEach = require('async/applyEach')

// const pull = require('pull-stream')


// const protocol = '/validity/0.0.1'
// const relayProtocol = '/relay/0.0.1'

// const curve = 'brainpoolP512t1'

// class MyNode extends libp2p {
//     constructor(_options) {
//         const defaults = {
//             modules: {
//                 transport: [ TCP ],
//                 streamMuxer: [ MPLEX ],
//                 connEncryption: [SECIO],
//                 dht: libp2pKadDHT
//             },
//             config: {
//                 dht: {
//                     kBucketSize: 20
//                 },
//                 EXPERIMENTAL: {
//                     // dht must be enabled
//                     dht: true
//                 }
//             }
//         }
//         super(defaultsDeep(_options, defaults))
//     }
// }
// function createNode(callback, addrs) {
//     let node

//     waterfall([
//         (cb) => PeerInfo.create(cb),
//         (peerInfo, cb) => {
//             addrs.forEach(addr => peerInfo.multiaddrs.add(addr))
//             node = new MyNode({
//                 peerInfo
//             })
//             node.start(cb)
//         }
//     ], (err) => callback(err, node))
// }
// process.stdin.setEncoding('utf8')

// parallel([
//     (cb) => createNode(cb, ['/ip4/0.0.0.0/tcp/0']),
//     (cb) => createNode(cb, ['/ip4/0.0.0.0/tcp/0']),
//     (cb) => createNode(cb, ['/ip4/0.0.0.0/tcp/0'])
// ], (err, nodes) => {
//     const node1 = nodes[0]
//     const node2 = nodes[1]
//     const node3 = nodes[2]

//     // nodes.forEach(node => registerRelayFunctionality(node))

//     // Bootstrapping ...
//     parallel([
//         (cb) => node1.dial(node2.peerInfo, cb),
//         (cb) => node2.dial(node3.peerInfo, cb),
//         (cb) => setTimeout(cb, 300)

//     ], (err) => {
//         if (err) { throw err }
//         let key = Buffer.from('/pk/0.0.0.0/tcp/0')
//         let value = Buffer.from('test //////////////// test')


//         // node1.dht.put(key, value, (err) => {
//         //     if (err) { throw err }
//         //     node1.dht.get(key, (err, data) => {
//         //         console.log(data.toString())
//         //     })
//         // })


//         node1.peerRouting.findPeer(node3.peerInfo.id, (err, peer) => {
//             if (err) { throw err }

//             console.log('Found it, multiaddrs are:')
//             peer.multiaddrs.forEach((ma) => console.log(ma.toString()))
//         })

//         // node1.dialProtocol(node2.peerInfo, constants.relayProtocol, (err, conn) => {
//         //     pull(
//         //         pull.values([node1.peerInfo.id.toBytes(), 'test']),
//         //         conn
//         //     )
//         // })
//     })








































//     // let cipher;

//     // node2.dialProtocol(node1.peerInfo, protocol, (err, conn) => {
//     //     pull(
//     //         pull.values(['test']),
//     //         conn
//     //     )
//     //     // Key exchange
//     //     // alice = crypto.createECDH(curve)
//     //     // let aliceKey = alice.generateKeys()
//     //     // pull(
//     //     //     pull.once(aliceKey),
//     //     //     conn,
//     //     //     pull.map(groupElement => {
//     //     //         let secret = bob.computeSecret(groupElement)
//     //     //         cipher = crypto.createCipheriv('aes-256-gcm', secret.slice(0,32), secret.slice(33))
//     //     //         return secret
//     //     //     }),
//     //     //     pull.onEnd()
//     //     //     // pull.collect((err, keyMaterial) => {
//     //     //     //     decipher = crypto.createDecipheriv('aes-256-gcm', keyMaterial.slice(0,32), keyMaterial.slice(33))
//     //     //     //     conn.getPeerInfo((err, peerInfo) => {
//     //     //     //         //decipher.setAAD(peerInfo.multiaddrs.toArray()[0].buffer)
//     //     //     //         console.log(peerInfo)
//     //     //     //     })
//     //     //     // }),
//     //     // ) 
//     // })

//     // // node2.dialProtocol(node1.peerInfo, protocol, (err, conn) => {
//     // //     pull(
//     // //         pull.once('test ... test '),
//     // //         cipher,
//     // //         conn
//     // //     )
//     // // })


//     // node1.handle(protocol, (err, conn) => {
//     //     pull(
//     //         conn,
//     //         pull.log()
//     //     )
//     //     // let decipher
//     //     // // Key exchange
//     //     // bob = crypto.createECDH(curve)
//     //     // let bobKey = bob.generateKeys()
//     //     // pull(
//     //     //     conn,
//     //     //     pull.map(groupElement => {
//     //     //         let secret = bob.computeSecret(groupElement)
//     //     //         decipher = crypto.createDecipheriv('aes-256-gcm', secret.slice(0,32), secret.slice(33))
//     //     //         return secret;
//     //     //     }),
//     //     //     pull.log()
//     //     // )
//     //     // pull(
//     //     //     pull.once(bobKey),
//     //     //     conn,
//     //     //     decipher,
//     //     //     pull.log()
//     //     // )
//     // })
//     // node1.handle(relayProtocol, relayer)
//     // node2.handle(relayProtocol, relayer)
//