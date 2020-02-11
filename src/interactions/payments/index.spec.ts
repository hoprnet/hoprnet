import dotenv from 'dotenv'
dotenv.config()
import assert from 'assert'

import { getPeerInfo, u8aEquals } from '../../utils'

import libp2p = require('libp2p')
import TCP = require('libp2p-tcp')
import MPLEX = require('libp2p-mplex')
import SECIO = require('libp2p-secio')

import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import { Interactions } from '..'

import Hopr from '../..'
import BN from 'bn.js'

describe('test payment (channel) interactions', function() {
  let Alice: Hopr<HoprCoreConnectorInstance>
  let Bob: Hopr<HoprCoreConnectorInstance>

  before(async function() {
    Alice = (await libp2p.create({
      peerInfo: await getPeerInfo({ id: 0 }),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnectorInstance>

    Bob = (await libp2p.create({
      peerInfo: await getPeerInfo({ id: 1 }),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnectorInstance>

    await Promise.all([
      /* prettier-ignore */
      Alice.start(),
      Bob.start()
    ])

    Alice.interactions = new Interactions(Alice)
    Bob.interactions = new Interactions(Bob)
  })

  it('should establish a connection and run through the opening sequence', async function() {
    const testArray = new Uint8Array(32).fill(0xff)
    const response = new Uint8Array(32).fill(0x00)

    Bob.paymentChannels = ({
      channel: {
        handleOpeningRequest(_: any) {
          return (source: any) => {
            return (async function*() {
              for await (const chunk of source) {
                assert(u8aEquals(Uint8Array.from(chunk.slice(0, 32)), testArray))

                yield response
              }
            })()
          }
        }
      }
    } as unknown) as HoprCoreConnectorInstance

    assert(
      u8aEquals(
        Uint8Array.from(
          await Alice.interactions.payments.open.interact(Bob.peerInfo, {
            balance: new BN(123456),
            balance_a: new BN(123),
            toU8a: () => testArray
          })
        ),
        response
      )
    )
  })

  // after(async function () {
  //   await Promise.all([
  //     Alice.stop(),
  //     Bob.stop()
  //   ])
  // })
})
