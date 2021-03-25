// import { expect } from 'chai'
import BN from 'bn.js'
import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { Channel, ChannelBalance, ChannelStatus, ChannelState } from './channel'
// import { Signature, Hash } from '.'
import Public from './public'
import SignedChannel from './signedChannel'
import { Balance } from '.'

chai.use(chaiAsPromised)

const generateChannelData = async () => {
  const balance = new ChannelBalance(undefined, {
    balance: new Balance(new BN(10)),
    balance_a: new Balance(new BN(2))
  })
  const state = new ChannelState(undefined, { state: ChannelStatus.UNINITIALISED })

  return new Channel(undefined, {
    state,
    balance
  })
}

describe('test signedChannel', function () {
  it('should create new signedChannel using struct', async function () {
    const channel = await generateChannelData()

    const noCounterparty = new SignedChannel(undefined, {
      channel
    })
    const withCounterparty = new SignedChannel(undefined, {
      channel,
      counterparty: new Public(stringToU8a('0x03767782fdb4564f0a2dee849d9fc356207dd89f195fcfd69ce0b02c6f03dfda40'))
    })

    expect(u8aToHex(await noCounterparty.signer)).to.equal(
      '0x000000000000000000000000000000000000000000000000000000000000000000'
    )
    expect(u8aToHex(await withCounterparty.signer)).to.equal(
      '0x03767782fdb4564f0a2dee849d9fc356207dd89f195fcfd69ce0b02c6f03dfda40'
    )
  })

  it('should error on verify', async function () {
    const channel = await generateChannelData()

    const signedChannel = new SignedChannel(undefined, {
      channel,
      counterparty: new Public(stringToU8a('0x03767782fdb4564f0a2dee849d9fc356207dd89f195fcfd69ce0b02c6f03dfda40'))
    })

    expect(signedChannel.verify(new Uint8Array())).to.eventually.throw
  })
})
