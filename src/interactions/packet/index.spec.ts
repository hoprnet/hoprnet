import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import libp2p = require('libp2p')
import TCP = require('libp2p-tcp')
import MPLEX = require('libp2p-mplex')
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'

import { Packet } from '../../messages/packet'
import Hopr from '../..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import HoprPolkadot from '@hoprnet/hopr-core-polkadot'
import { SRMLTypes, Types, Active, ChannelBalance, AccountId, Hash, SignedChannel } from '@hoprnet/hopr-core-polkadot/lib/srml_types'
import { ApiPromise } from '@polkadot/api'
import { waitReady } from '@polkadot/wasm-crypto'
import Keyring from '@polkadot/keyring'
import { TypeRegistry } from '@polkadot/types'
import { Interactions } from '..'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import BN from 'bn.js'
import HoprPolkadotClass from '@hoprnet/hopr-core-polkadot/lib'

describe('check packet forwarding & acknowledgement generation', function() {
  const channels = new Map<string, SignedChannel>()
  const states = new Map<string, any>()

  const typeRegistry = new TypeRegistry()

  typeRegistry.register(SRMLTypes)

  async function generateNode(): Promise<Hopr<HoprPolkadotClass>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprPolkadot>

    // @ts-ignore
    node.peerInfo.multiaddrs.add('/ip4/0.0.0.0/tcp/0')

    await Promise.all([
      /* prettier-ignore */
      node.start(),
      waitReady()
    ])

    node.peerRouting.findPeer = (_: PeerId) => Promise.reject('not implemented')

    node.interactions = new Interactions(node)

    const kPair = new Keyring({ type: 'sr25519' }).addFromSeed(node.peerInfo.id.pubKey.marshal().slice(0, 32), undefined, 'sr25519')
    node.paymentChannels = new HoprPolkadot(
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
      LevelUp(Memdown())
    )

    await node.paymentChannels.start()
    await node.paymentChannels.initOnchainValues()

    node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)

    return (node as unknown) as Hopr<HoprPolkadotClass>
  }

  it('should forward a packet', async function() {
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

    const signature = await Bob.paymentChannels.utils.sign(channel.toU8a(), Bob.peerInfo.id.privKey.marshal(), Bob.peerInfo.id.pubKey.marshal())

    const channelId = await Alice.paymentChannels.utils.getId(
      new AccountId(typeRegistry, Alice.paymentChannels.self.keyPair.publicKey),
      new AccountId(typeRegistry, Bob.paymentChannels.self.keyPair.publicKey),
      Bob.paymentChannels.api
    )

    const channelIdSecond = await Bob.paymentChannels.utils.getId(
      new AccountId(typeRegistry, Bob.paymentChannels.self.keyPair.publicKey),
      new AccountId(typeRegistry, Chris.paymentChannels.self.keyPair.publicKey),
      Bob.paymentChannels.api
    )

    const channelRecord = new Types.SignedChannel(undefined, {
      channel,
      signature
    })

    channels.set(channelIdSecond.toHex(), channelRecord)
    channels.set(channelId.toHex(), channelRecord)

    await Promise.all([
      Alice.paymentChannels.db.put(
        Alice.paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, Bob.paymentChannels.self.keyPair.publicKey)),
        channelRecord.toU8a()
      ),
      Bob.paymentChannels.db.put(
        Bob.paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, Alice.paymentChannels.self.keyPair.publicKey)),
        channelRecord.toU8a()
      ),
      Bob.paymentChannels.db.put(
        Bob.paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, Chris.paymentChannels.self.keyPair.publicKey)),
        channelRecord.toU8a()
      ),
      Chris.paymentChannels.db.put(
        Chris.paymentChannels.dbKeys.Channel(new AccountId(typeRegistry, Bob.paymentChannels.self.keyPair.publicKey)),
        channelRecord.toU8a()
      )
    ])

    const testArray = new Uint8Array([1, 2, 3, 4])

    // const packet = await Packet.create(Alice, new TextEncoder().encode('123'), [Bob.peerInfo.id, Chris.peerInfo.id])

    // const bobsPacket = new Packet(Bob, {
    //   bytes: packet.buffer,
    //   offset: packet.byteOffset
    // })

    // console.log(`before`, u8aToHex(bobsPacket))
    // await bobsPacket.forwardTransform()
    // console.log(`after`, u8aToHex(bobsPacket))

    await Alice.interactions.packet.forward.interact(
      Bob.peerInfo,
      await Packet.create(Alice, new TextEncoder().encode('123'), [Bob.peerInfo.id, Chris.peerInfo.id])
    )
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
