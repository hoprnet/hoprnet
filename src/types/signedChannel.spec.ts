import assert from 'assert'
import BN from 'bn.js'
import { stringToU8a, randomInteger } from '@hoprnet/hopr-utils'
import { Channel, SignedChannel, ChannelBalance, Signature, Hash, ChannelStatus } from '.'
import * as utils from '../utils'
import * as testconfigs from '../config.spec'

const [userA] = testconfigs.DEMO_ACCOUNTS.map((str: string) => stringToU8a(str))

const generateChannelData = async () => {
  const balance = new ChannelBalance(undefined, {
    balance: new BN(10),
    balance_a: new BN(2),
  })
  const status = ChannelStatus.UNINITIALISED

  return {
    balance,
    status,
  }
}

describe('test signedChannel construction', async function () {
  const userAPubKey = await utils.privKeyToPubKey(userA)

  it('should create new signedChannel using struct', async function () {
    const channelData = await generateChannelData()

    const channel = new Channel(undefined, channelData)

    const signedChannel = new SignedChannel(undefined, {
      channel,
    })

    await channel.sign(userA, undefined, {
      bytes: signedChannel.buffer,
      offset: signedChannel.signatureOffset,
    })

    assert(await signedChannel.verify(userAPubKey), 'signature must be valid')

    assert(new Hash(await signedChannel.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedChannel.channel.balance.eq(channelData.balance), 'wrong balance')
    assert(new BN(signedChannel.channel.status).eq(new BN(channelData.status)), 'wrong status')
  })

  it('should create new signedChannel using array', async function () {
    const channelData = await generateChannelData()

    const channel = new Channel(undefined, channelData)
    const signature = new Signature()

    await channel.sign(userA, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset,
    })

    const signedChannelA = new SignedChannel(undefined, {
      signature,
      channel,
    })
    const signedChannelB = new SignedChannel({
      bytes: signedChannelA.buffer,
      offset: signedChannelA.byteOffset,
    })

    assert(await signedChannelA.verify(userAPubKey), 'signature must be valid')
    assert(new Hash(await signedChannelA.signer).eq(userAPubKey), 'signer incorrect')

    assert(await signedChannelB.verify(userAPubKey), 'signature must be valid')
    assert(new Hash(await signedChannelB.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedChannelB.channel.balance.eq(channelData.balance), 'wrong balance')
    assert(new BN(signedChannelB.channel.status).eq(new BN(channelData.status)), 'wrong status')
  })

  it('should create new signedChannel out of continous memory', async function () {
    const channelData = await generateChannelData()

    const channel = new Channel(undefined, channelData)

    const signature = new Signature()

    await channel.sign(userA, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset,
    })

    const offset = randomInteger(1, 31)

    const array = new Uint8Array(SignedChannel.SIZE + offset).fill(0x00)

    const signedChannel = new SignedChannel(
      {
        bytes: array.buffer,
        offset: array.byteOffset + offset,
      },
      {
        channel,
        signature,
      }
    )

    assert(await signedChannel.verify(userAPubKey), 'signature must be valid')

    assert(new Hash(await signedChannel.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedChannel.channel.balance.eq(channelData.balance), 'wrong balance')
    assert(new BN(signedChannel.channel.status).eq(new BN(channelData.status)), 'wrong status')
  })
})
