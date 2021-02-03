import Multiaddr from 'multiaddr'

const testsTransport = require('libp2p-interfaces/src/transport/tests')
// const testsDiscovery = require('libp2p-interfaces/src/peer-discovery/tests')

import HoprConnect from '../src'
import PeerId from 'peer-id'

import libp2p, { Handler, MultiaddrConnection, Upgrader } from 'libp2p'

const SECIO = require('libp2p-secio')
const MPLEX = require('libp2p-mplex')
const Upgrader = require('libp2p/src/upgrader')

import { Alice, Bob, Charly, Dave } from '../examples/identities'

async function startBootstrapServer(privKey: Uint8Array, port: number): Promise<libp2p> {
  const node = await libp2p.create({
    peerId: await PeerId.createFromPrivKey(privKey),
    addresses: {
      listen: [Multiaddr(`/ip4/0.0.0.0/tcp/${port}`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO]
    }
  })

  await node.start()

  return node
}

async function myUpgrader(pId: PeerId): Promise<Upgrader> {
  return new Upgrader({
    localPeer: pId,
    metrics: undefined,
    cryptos: new Map([[SECIO.protocol, SECIO]]),
    muxers: new Map([[MPLEX.multicodec, MPLEX]])
  })
}

async function startClient(privKey: Uint8Array, port: number, bootstrapAddress: Multiaddr): Promise<libp2p> {
  const node = await libp2p.create({
    peerId: await PeerId.createFromPrivKey(privKey),
    addresses: {
      listen: [Multiaddr(`/ip4/0.0.0.0/tcp/${port}`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO]
    },
    config: {
      transport: {
        HoprConnect: {
          bootstrapServers: [bootstrapAddress],
          // simulates a NAT
          // do NOT use this in production
          __noDirectConnections: true
        }
      }
    }
  })

  await node.start()

  return node
}

describe('libp2p compliance', () => {
  testsTransport({
    async setup(options: { upgrader: Upgrader }) {
      await startBootstrapServer(Alice, 9092)
      // const bob = await startClient(
      //   Bob,
      //   9093,
      //   Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`)
      // )
      // await startClient(Charly, 9094, Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`))

      //let transport = bob.transportManager._transports.get('HoprConnect')

      const client = await startClient(
        Charly,
        9095,
        Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`)
      )

      await client.dial(Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`))

      const addrs = [
        Multiaddr(`/ip4/127.0.0.1/tcp/9093/p2p/${await PeerId.createFromPrivKey(Bob)}`),
        Multiaddr(`/p2p/${await PeerId.createFromPrivKey(Dave)}`)
      ]

      const upgrader = await myUpgrader(await PeerId.createFromPrivKey(Bob))
      const transport = new HoprConnect({
        upgrader: {
          async upgradeInbound(maConn: MultiaddrConnection) {
            if (maConn.remoteAddr.getPeerId() === (await PeerId.createFromPrivKey(Alice)).toB58String()) {
              return upgrader.upgradeInbound(maConn)
            } else {
              return options.upgrader.upgradeInbound(maConn)
            }
          },
          async upgradeOutbound(maConn: MultiaddrConnection) {
            if (maConn.remoteAddr.getPeerId() === (await PeerId.createFromPrivKey(Alice)).toB58String()) {
              return upgrader.upgradeOutbound(maConn)
            } else {
              return options.upgrader.upgradeOutbound(maConn)
            }
          },
          protocols: upgrader.protocols
        },
        libp2p: {
          peerId: await PeerId.createFromPrivKey(Bob),
          handle: (protocols: string | string[], handler: Handler) => {
            protocols = Array.isArray(protocols) ? protocols : [protocols]
            protocols.forEach((protocol: string) => {
              upgrader.protocols.set(protocol, handler)
            })
          },
          registrar: {
            getConnection() {}
          },
          dialer: {
            async connectToPeer(_pId: PeerId) {
              return transport.dial(Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`))
            },
            _pendingDials: []
          },
          connectionManager: {
            connections: new Map()
          },
          multiaddrs: []
        } as any,
        bootstrapServers: [Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Alice)}`)],
        __noDirectConnections: true
      })

      //const network = require('my-network-lib')
      //const connect = network.connect
      // const connector = {
      //   delay(delayMs) {
      //     // Add a delay in the connection mechanism for the transport
      //     // (this is used by the dial tests)
      //     network.connect = (...args) => setTimeout(() => connect(...args), delayMs)
      //   },
      //   restore() {
      //     // Restore the connection mechanism to normal
      //     network.connect = connect
      //   }
      // }

      return { transport, addrs }
    },
    teardown() {
      // Clean up any resources created by setup()
    }
  })
})

// describe('interface-discovery compliance', () => {
//   let intervalId

//   testsDiscovery({
//     setup() {
//       const mockUpgrader = {
//         upgradeInbound: (maConn) => maConn,
//         upgradeOutbound: (maConn) => maConn
//       }
//       const ws = new WStar({ upgrader: mockUpgrader, wrtc: wrtc })
//       const maStr = '/ip4/127.0.0.1/tcp/15555/ws/p2p-webrtc-star/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSooo2d'

//       intervalId = setInterval(() => ws._peerDiscovered(maStr), 1000)

//       return ws.discovery
//     },
//     teardown() {
//       clearInterval(intervalId)
//     }
//   })
// })
