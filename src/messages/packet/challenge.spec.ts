import assert from 'assert'
import { Challenge } from './challenge'
import { Utils, Constants } from '@hoprnet/hopr-core-polkadot'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { HoprCoreConnectorClass } from '@hoprnet/hopr-core-connector-interface'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'

describe('test creation & verification of a challenge', function() {
  it('should create a verifiable challenge', async function() {
    const paymentChannels = ({
      utils: Utils,
      constants: Constants
    } as unknown) as HoprCoreConnectorClass

    const hash = await paymentChannels.utils.hash(new Uint8Array(32))

    const exponent = randomBytes(32)
    
    const challenge = Challenge.create(paymentChannels, secp256k1.publicKeyCreate(exponent), new BN(0))

    const peerId = await PeerId.create({
      keyType: 'secp256k1'
    })
    await challenge.sign(peerId)

    assert(challenge.verify(peerId))
  })
})
