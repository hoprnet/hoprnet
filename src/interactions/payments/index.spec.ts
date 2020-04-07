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
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { PaymentInteractions } from '.'
import { privKeyToPeerId } from '../../utils'

import type Hopr from '../..'
import type { HoprOptions } from '../..'

import BN from 'bn.js'

async function generateNode(id: number): Promise<Hopr<HoprCoreConnector>> {
  const peerId = await privKeyToPeerId(config.DEMO_ACCOUNTS[id])
  const node = (await libp2p.create({
    peerInfo: await getPeerInfo({ id, peerId } as HoprOptions),
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO]
    }
  })) as Hopr<HoprCoreConnector>

  await node.start()

  node.paymentChannels = ({
    types: Types,
    self: {
      privateKey: peerId.privKey.marshal(),
      publicKey: peerId.pubKey.marshal()
    }
  } as unknown) as HoprCoreConnector

  node.interactions = {
    payments: new PaymentInteractions(node)
  } as Hopr<HoprCoreConnector>['interactions']

  return node
}

describe('test payment (channel) interactions', function() {
  it('should establish a connection and run through the opening sequence', async function() {
    const [Alice, Bob] = await Promise.all([generateNode(0), generateNode(1)])
    const testArray = new Uint8Array(32).fill(0xff)
    const response = new Uint8Array(Types.SignedChannel.SIZE).fill(0x00)

    Bob.paymentChannels = ({
      channel: {
        handleOpeningRequest(_: any) {
          return (source: any) => {
            return (async function*() {
              for await (const chunk of source) {
                assert(chunk.length > 0, 'Should receive a message')

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

    await Promise.all([
      Alice.stop(),
      Bob.stop()
    ])
  })
})
