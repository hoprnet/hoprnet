import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'
import { encode, decode } from 'rlp'

import { Packet } from '../../messages/packet'
import Hopr from '../..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { PacketInteractions } from '.'
import { OnChainKey } from '../payments/onChainKey'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import BN from 'bn.js'
import { randomBytes, createHash } from 'crypto'
import * as DbKeys from '../../dbKeys'
import { stringToU8a, u8aEquals, randomInteger, u8aConcat } from '@hoprnet/hopr-utils'
import secp256k1 from 'secp256k1'

import { MAX_HOPS } from '../../constants'

import assert from 'assert'
import Multiaddr from 'multiaddr'

describe('check packet forwarding & acknowledgement generation', function () {
  async function generateNode(): Promise<Hopr<HoprCoreConnector>> {
    const db = LevelUp(Memdown())

    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
      },
    })) as Hopr<HoprCoreConnector>

    node.db = db

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

    node.peerRouting.findPeer = (_: PeerId): Promise<never> => {
      return Promise.reject(Error('not implemented'))
    }

    node.interactions = {
      packet: new PacketInteractions(node),
      payments: {
        onChainKey: new OnChainKey(node),
      },
    } as Hopr<HoprCoreConnector>['interactions']

    node.paymentChannels = ({
      start() {
        return Promise.resolve()
      },
      initOnchainValues() {
        return Promise.resolve()
      },
      stop() {
        return Promise.resolve()
      },
      channel: {
        create() {
          return {
            ticket: {
              create(
                _channel: any,
                _amount: any,
                challenge: Uint8Array,
                arr: {
                  bytes: ArrayBuffer
                  offset: number
                }
              ) {
                const ticket = new Uint8Array(arr.bytes, arr.offset, 33 + 32)

                ticket.set(node.peerInfo.id.pubKey.marshal())
                ticket.set(challenge, 33)

                // @ts-ignore
                ticket.signer = ticket.subarray(0, 33)

                // @ts-ignore
                ticket.ticket = {
                  challenge: ticket.subarray(33, 33 + 32),
                }

                return ticket
              },
            },
          }
        },
        isOpen() {
          return Promise.resolve(true)
        },
      },
      utils: {
        hash(msg: Uint8Array) {
          return Promise.resolve(createHash('sha256').update(msg).digest())
        },
        async sign(msg: Uint8Array, privKey: Uint8Array) {
          const signature = secp256k1.ecdsaSign(msg, privKey)

          const result = u8aConcat(signature.signature, new Uint8Array([signature.recid]))

          // @ts-ignore
          result.signature = result.subarray(0, 64)
          // @ts-ignore
          result.recovery = result.subarray(64, 65)[0]
          return result
        },
        verify() {
          return true
        },
        pubKeyToAccountId(arr: Uint8Array) {
          return arr
        },
        getId(a: Uint8Array, b: Uint8Array) {
          if (Buffer.compare(a, b)) {
            return createHash('sha256').update(u8aConcat(a, b)).digest()
          } else {
            return createHash('sha256').update(u8aConcat(b, a)).digest()
          }
        },
      },
      types: {
        SignedTicket: {
          SIZE: 33 + 32,
          create(arr: { bytes: ArrayBuffer; offset: number }) {
            const ticket = new Uint8Array(arr.bytes, arr.offset, 33 + 32)

            // @ts-ignore
            ticket.signer = ticket.subarray(0, 33)
            // @ts-ignore
            ticket.ticket = {
              getEmbeddedFunds() {
                return new BN(1)
              },
              challenge: ticket.subarray(33, 33 + 32),
            }
            return ticket
          },
        },
        Signature: {
          SIZE: 65,
          create(arr: { bytes: ArrayBuffer; offset: number }) {
            const signature = new Uint8Array(arr.bytes, arr.offset, 65)

            // @ts-ignore
            signature.signature = signature.subarray(0, 64)
            // @ts-ignore
            signature.recovery = signature.subarray(64, 65)[0]

            return signature
          },
        },
        ChannelBalance: {
          create() {
            return new Uint8Array(32).fill(0x00)
          },
        },
      },
    } as unknown) as HoprCoreConnector

    node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)
    node.dbKeys = DbKeys

    await node.start()

    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should forward a packet and receive aknowledgements', async function () {
    const nodes = await Promise.all([
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
    ])

    connectionHelper(nodes)

    let emitPromises: Promise<any>[]

    const testMsg = randomBytes(randomInteger(37, 131))

    for (let i = 1; i <= MAX_HOPS; i++) {
      emitPromises = []

      for (let j = 0; j < i; j++) {
        emitPromises.push(emitCheckerHelper(nodes[j], nodes[j + 1].peerInfo.id))
      }

      nodes[i].output = (arr: Uint8Array) => {
        const [msg] = (decode(Buffer.from(arr)) as unknown) as Buffer[]

        assert(u8aEquals(msg, testMsg), `Checks that we receive the right message.`)
      }

      await nodes[0].interactions.packet.forward.interact(
        nodes[1].peerInfo,
        await Packet.create(
          nodes[0],
          encode([testMsg, new TextEncoder().encode(Date.now().toString())]),
          nodes.slice(1, i + 1).map((node: Hopr<HoprCoreConnector>) => node.peerInfo.id)
        )
      )

      try {
        await Promise.all(emitPromises)
      } catch (err) {
        assert.fail(`Checks that we emit an event once we got an acknowledgement.`)
      }
    }

    await Promise.all(nodes.map((node: Hopr<HoprCoreConnector>) => node.stop()))
  })
})

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper<Chain extends HoprCoreConnector>(nodes: Hopr<Chain>[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}

/**
 * Returns a Promise that resolves once the acknowledgement has been received
 * @param node our Hopr node
 * @param sender the sender of the packet
 */
function emitCheckerHelper<Chain extends HoprCoreConnector>(
  node: Hopr<Chain>,
  sender: PeerId
): Promise<any> {
  return new Promise<any>((resolve, reject) => {
    node.interactions.packet.acknowledgment.emit = (event: string) => {
      node.dbKeys.UnAcknowledgedTicketsParse(stringToU8a(event)).then(([counterparty]) => {
        if (u8aEquals(sender.pubKey.marshal(), counterparty.pubKey.marshal())) {
          resolve()
        } else {
          reject()
        }
      }, reject)

      return false
    }
  })
}

// async function channelDbHelper<Chain extends HoprCoreConnector>(
//   typeRegistry: TypeRegistry,
//   record: Uint8Array,
//   ...nodes: Hopr<Chain>[]
// ): Promise<void> {
//   const promises: Promise<any>[] = []

//   if (nodes.length < 2) {
//     throw Error('cannot do this with less than two nodes.')
//   }

//   promises.push(
//     nodes[0].db.put(
//       Buffer.from(
//         nodes[0].paymentChannels.dbKeys.Channel(
//           new AccountId(typeRegistry, nodes[1].paymentChannels.self.onChainKeyPair.publicKey)
//         )
//       ),
//       Buffer.from(record)
//     )
//   )

//   for (let i = 1; i < nodes.length - 1; i++) {
//     promises.push(
//       nodes[i].db
//         .batch()
//         .put(
//           Buffer.from(
//             nodes[i].paymentChannels.dbKeys.Channel(
//               new AccountId(
//                 typeRegistry,
//                 nodes[i - 1].paymentChannels.self.onChainKeyPair.publicKey
//               )
//             )
//           ),
//           Buffer.from(record)
//         )
//         .put(
//           Buffer.from(
//             nodes[i].paymentChannels.dbKeys.Channel(
//               new AccountId(
//                 typeRegistry,
//                 nodes[i + 1].paymentChannels.self.onChainKeyPair.publicKey
//               )
//             )
//           ),
//           Buffer.from(record)
//         )
//         .write()
//     )
//   }

//   await Promise.all(promises)
// }

// function getIds<Chain extends HoprCoreConnector>(
//   typeRegistry: TypeRegistry,
//   ...nodes: Hopr<Chain>[]
// ) {
//   const promises: Promise<any>[] = []
//   for (let i = 0; i < nodes.length - 1; i++) {
//     promises.push(
//       nodes[i].paymentChannels.utils.getId(
//         new AccountId(typeRegistry, nodes[i].paymentChannels.self.onChainKeyPair.publicKey),
//         new AccountId(typeRegistry, nodes[i + 1].paymentChannels.self.onChainKeyPair.publicKey)
//       )
//     )
//   }

//   return Promise.all(promises)
// }
