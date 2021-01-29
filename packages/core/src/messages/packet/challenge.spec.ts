import assert from 'assert'
import { Challenge } from './challenge'
import { Utils, Types } from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'
import PeerId from 'peer-id'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { randomBytes } from 'crypto'
import { randomInteger, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

describe('test creation & verification of a challenge', function () {
  it('should create a verifiable challenge', async function () {
    const paymentChannels = ({
      utils: Utils,
      types: new Types()
    } as unknown) as HoprCoreConnector

    for (let i = 0; i < 10; i++) {
      const secret = randomBytes(32)

      const peerId = await PeerId.create({
        keyType: 'secp256k1'
      })

      const challenge = await Challenge.create(paymentChannels, secret, new BN(0)).sign(peerId)

      assert(await challenge.verify(peerId), `Previously generated signature should be valid.`)

      assert(u8aEquals(await challenge.counterparty, peerId.pubKey.marshal()), `recovered pubKey should be equal.`)

      let exponent = randomInteger(0, 7)
      let index = randomInteger(0, challenge.length - 1)

      challenge[index] = challenge[index] ^ (1 << exponent)

      if (await challenge.verify(peerId)) {
        // @TODO change to assert.fail
        console.log(`found invalid signature, <${u8aToHex(challenge)}>, byte #${index}, bit #${exponent}`)
      }
    }
  })
})
