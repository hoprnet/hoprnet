import dotenv from 'dotenv'
dotenv.config()
import assert from 'assert'

import { getPeerInfo } from '../../utils'

// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import { Types } from '@hoprnet/hopr-core-polkadot'
import * as config from '@hoprnet/hopr-core-polkadot/lib/config'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Interactions } from '..'
import { privKeyToPeerId } from '../../utils'

import Hopr from '../..'
import BN from 'bn.js'

describe('test payment (channel) interactions', function() {
  let Alice: Hopr<HoprCoreConnector>
  let Bob: Hopr<HoprCoreConnector>

  before(async function() {
    const peerIdAlice = await privKeyToPeerId(config.DEMO_ACCOUNTS[0])
    Alice = (await libp2p.create({
      peerInfo: await getPeerInfo({ id: 0, peerId: peerIdAlice }),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnector>

    const peerIdBob = await privKeyToPeerId(config.DEMO_ACCOUNTS[1])
    Bob = (await libp2p.create({
      peerInfo: await getPeerInfo({ id: 1, peerId: peerIdBob }),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnector>

    await Promise.all([
      /* prettier-ignore */
      Alice.start(),
      Bob.start()
    ])

    Alice.paymentChannels = ({
      types: Types,
      self: {
        privateKey: peerIdAlice.privKey.marshal(),
        publicKey: peerIdAlice.pubKey.marshal()
      }
    } as unknown) as HoprCoreConnector

    Bob.paymentChannels = ({
      types: Types,
      self: {
        privateKey: peerIdBob.privKey.marshal(),
        publicKey: peerIdBob.pubKey.marshal()
      }
    } as unknown) as HoprCoreConnector

    Alice.interactions = new Interactions(Alice)
    Bob.interactions = new Interactions(Bob)
  })

  it('should establish a connection and run through the opening sequence', async function() {
    const testArray = new Uint8Array(32).fill(0xff)
    const response = new Uint8Array(Types.SignedChannel.SIZE).fill(0x00)

    Bob.paymentChannels = ({
      channel: {
        handleOpeningRequest(_: any) {
          return (source: any) => {
            return (async function*() {
              for await (const chunk of source) {
                assert(chunk.length > 0, 'Should receive a message')
                console.log(chunk.slice(0, 32))

                console.log('sending')
                yield response.slice()
              }
            })()
          }
        }
      }
    } as unknown) as HoprCoreConnector

    assert(
      (await Alice.interactions.payments.open.interact(Bob.peerInfo, {
        balance: new BN(123456),
        balance_a: new BN(123),
        toU8a: () => testArray
      })) != null,
      'Should a receive a message from counterparty'
    )
  })

  after(async function() {
    await Promise.all([
      /* prettier-ignore */
      Alice != null ? Alice.stop() : Promise.resolve(),
      Bob != null ? Bob.stop() : Promise.resolve()
    ])
  })
})
