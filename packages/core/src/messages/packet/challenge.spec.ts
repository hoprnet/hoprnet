import assert from 'assert'
import {Challenge} from './challenge'
import {Utils, Types} from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'
import PeerId from 'peer-id'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import {randomBytes} from 'crypto'
import {u8aEquals} from '@hoprnet/hopr-utils'

describe('test creation & verification of a challenge', function () {
  it('should create a verifiable challenge', async function () {
    const paymentChannels = ({
      utils: Utils,
      types: new Types()
    } as unknown) as HoprCoreConnector

    const secret = randomBytes(32)

    const peerId = await PeerId.create({
      keyType: 'secp256k1'
    })

    const challenge = await Challenge.create(paymentChannels, secret, new BN(0)).sign(peerId)

    assert(await challenge.verify(peerId), `Previously generated signature should be valid.`)

    assert(u8aEquals(await challenge.counterparty, peerId.pubKey.marshal()), `recovered pubKey should be equal.`)

    challenge[0] ^= 0xff
    try {
      await challenge.verify(peerId)
      assert.fail(`Manipulated signature should be with high probability invalid.`)
    } catch {}
  })
})
