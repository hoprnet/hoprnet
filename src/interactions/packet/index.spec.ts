import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import libp2p = require('libp2p')
import TCP = require('libp2p-tcp')
import MPLEX = require('libp2p-mplex')
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'
import { encode, decode } from 'rlp'

import { Packet } from '../../messages/packet'
import Hopr from '../..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import { SRMLTypes, Types, Active, ChannelBalance, AccountId, Hash, SignedChannel } from '@hoprnet/hopr-core-polkadot/lib/srml_types'
import { ApiPromise } from '@polkadot/api'
import { waitReady } from '@polkadot/wasm-crypto'
import Keyring from '@polkadot/keyring'
import { TypeRegistry } from '@polkadot/types'
import { Interactions } from '..'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import BN from 'bn.js'
import HoprPolkadotClass from '@hoprnet/hopr-core-polkadot'
import { randomBytes } from 'crypto'
import { DbKeys } from '../../db_keys'
import { stringToU8a, u8aEquals, u8aToHex } from '../../utils'

import assert from 'assert'
import Multiaddr from 'multiaddr'

describe('check packet forwarding & acknowledgement generation', function() {
  const channels = new Map<string, SignedChannel>()
  const states = new Map<string, any>()

  const typeRegistry = new TypeRegistry()

  typeRegistry.register(SRMLTypes)

  async function generateNode(): Promise<Hopr<HoprPolkadotClass>> {
    const db = LevelUp(Memdown())

    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprPolkadotClass>

    node.db = db

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

    await Promise.all([
      /* prettier-ignore */
      node.start(),
      waitReady()
    ])

    node.peerRouting.findPeer = (_: PeerId): Promise<never> => {
      return Promise.reject(Error('not implemented'))
    }

    node.interactions = new Interactions(node)

    const kPair = new Keyring({ type: 'sr25519' }).addFromSeed(node.peerInfo.id.pubKey.marshal().slice(0, 32), undefined, 'sr25519')
    node.paymentChannels = new HoprPolkadotClass(
      ({
        isReady: Promise.resolve(true),
        query: {
          hopr: {
            channels(channelId: Hash) {
              if (!channels.has(channelId.toHex())) {
                throw Error(`missing channel ${channelId.toHex()}`)
              }

              return Promise.resolve(channels.get(channelId.toHex()))
            },
            state(accountId: AccountId) {
              if (!states.has(accountId.toHex())) {
                throw Error(`party ${accountId.toHex()} has not set any on-chain secrets.`)
              }

              return Promise.resolve(states.get(accountId.toHex()))
            }
          },
          system: {
            events(_handler: () => void) {},
            accountNonce() {
              return Promise.resolve({
                toNumber: () => 0
              })
            }
          }
        },
        tx: {
          hopr: {
            init(secret: any, publicKey: any) {
              const signAndSend = (keyPair: any) => {
                states.set(new AccountId(typeRegistry, keyPair.publicKey).toHex(), {
                  secret,
                  publicKey
                })

                return Promise.resolve()
              }

              return { signAndSend }
            }
          }
        },
        createType(name: string, ...args: any[]) {
          const result = new (typeRegistry.get(name))(typeRegistry, ...args)

          return result
        },
        registry: typeRegistry
      } as unknown) as ApiPromise,
      {
        publicKey: node.peerInfo.id.pubKey.marshal(),
        privateKey: node.peerInfo.id.privKey.marshal(),
        keyPair: kPair
      },
      db
    )

    await node.paymentChannels.start()
    await node.paymentChannels.initOnchainValues()

    node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)
    node.dbKeys = new DbKeys()

    return (node as unknown) as Hopr<HoprPolkadotClass>
  }

  it('should forward a packet and receive aknowledgements', async function() {
    const [Alice, Bob, Chris, Dave] = await Promise.all([generateNode(), generateNode(), generateNode(), generateNode()])

    connectionHelper(Alice, Bob, Chris, Dave)

    const channel = new Types.Channel(
      typeRegistry,
      new Active(
        typeRegistry,
        new ChannelBalance(typeRegistry, {
          balance: new BN(12345),
          balance_a: new BN(123)
        })
      )
    )

    const [channelId, channelIdSecond, channelIdThird] = await getIds(typeRegistry, Alice, Bob, Chris, Dave)

    const channelRecord = new Types.SignedChannel(undefined, {
      channel,
      signature: await Bob.paymentChannels.utils.sign(channel.toU8a(), Bob.peerInfo.id.privKey.marshal(), Bob.peerInfo.id.pubKey.marshal())
    })

    channels.set(channelIdThird.toHex(), channelRecord)
    channels.set(channelIdSecond.toHex(), channelRecord)
    channels.set(channelId.toHex(), channelRecord)

    await channelDbHelper(typeRegistry, channelRecord, Alice, Bob, Chris, Dave)

    const testMsg = randomBytes(73)

    const emitPromises: Promise<any>[] = []
    emitPromises.push(emitCheckerHelper(Alice, Bob.peerInfo.id))

    emitPromises.push(emitCheckerHelper(Bob, Chris.peerInfo.id))

    Chris.output = (arr: Uint8Array) => {
      const [msg] = (decode(Buffer.from(arr)) as unknown) as Buffer[]

      assert(u8aEquals(msg, testMsg), `Checks that we receive the right message.`)
    }

    await Alice.interactions.packet.forward.interact(
      Bob.peerInfo,
      await Packet.create(Alice, encode([testMsg, new TextEncoder().encode(Date.now().toString())]), [Bob.peerInfo.id, Chris.peerInfo.id])
    )

    const testMsgSecond = randomBytes(101)

    Dave.output = (arr: Uint8Array) => {
      const [msg] = (decode(Buffer.from(arr)) as unknown) as Buffer[]

      assert(u8aEquals(msg, testMsgSecond), `Checks that we receive the right message.`)
    }

    emitPromises.push(emitCheckerHelper(Chris, Dave.peerInfo.id))

    await Alice.interactions.packet.forward.interact(
      Bob.peerInfo,
      await Packet.create(Alice, encode([testMsgSecond, new TextEncoder().encode(Date.now().toString())]), [
        Bob.peerInfo.id,
        Chris.peerInfo.id,
        Dave.peerInfo.id
      ])
    )

    assert.doesNotReject(Promise.all(emitPromises), `Checks that we emit an event once we got an acknowledgement.`)
  })

  // afterEach(function() {
  //   channels.clear()
  // })
})

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper<Chain extends HoprCoreConnectorInstance>(...nodes: Hopr<Chain>[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}

function emitCheckerHelper<Chain extends HoprCoreConnectorInstance>(node: Hopr<Chain>, sender: PeerId): Promise<any> {
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

function channelDbHelper<Chain extends HoprCoreConnectorInstance>(typeRegistry: TypeRegistry, record: Uint8Array, ...nodes: Hopr<Chain>[]): Promise<any> {
  const promises: Promise<any>[] = []

  if (nodes.length < 2) {
    throw Error('cannot do this with less than two nodes.')
  }

  promises.push(nodes[0].db.put(u8aToHex(nodes[0].paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, nodes[1].paymentChannels.self.keyPair.publicKey))), record))

  for (let i = 1; i < nodes.length - 1; i++) {
    promises.push(
      nodes[i].db
        .batch()
        .put(u8aToHex(nodes[i].paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, nodes[i - 1].paymentChannels.self.keyPair.publicKey))), record)
        .put(u8aToHex(nodes[i].paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, nodes[i + 1].paymentChannels.self.keyPair.publicKey))), record)
        .write()
    )
  }

  return Promise.all(promises)
}

function getIds<Chain extends HoprCoreConnectorInstance>(typeRegistry: TypeRegistry, ...nodes: Hopr<Chain>[]) {
  const promises: Promise<any>[] = []
  for (let i = 0; i < nodes.length - 1; i++) {
    promises.push(
      nodes[i].paymentChannels.utils.getId(
        new AccountId(typeRegistry, nodes[i].paymentChannels.self.keyPair.publicKey),
        new AccountId(typeRegistry, nodes[i + 1].paymentChannels.self.keyPair.publicKey),
        // @ts-ignore
        nodes[i].paymentChannels.api
      )
    )
  }

  return Promise.all(promises)
}
