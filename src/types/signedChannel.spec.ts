import assert from 'assert'
import BN from 'bn.js'
import { Channel, SignedChannel, ChannelBalance, Signature, Hash } from '.'
import { ChannelStatus } from './channel'
import * as u8a from '../core/u8a'
import * as utils from '../utils'
import { DEMO_ACCOUNTS } from '../config'

const [userA] = DEMO_ACCOUNTS.map(str => u8a.stringToU8a(str))

const generateChannelData = async () => {
  const balance = new ChannelBalance(undefined, {
    balance: new BN(10),
    balance_a: new BN(2)
  })
  const status = ChannelStatus.UNINITIALISED

  return {
    balance,
    status
  }
}

describe('test signedChannel construction', function() {
  it('should create new signedChannel using struct', async function() {
    const channelData = await generateChannelData()

    const channel = new Channel(undefined, channelData)
    const signature = await utils.sign(await channel.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedChannel = new SignedChannel(undefined, {
      signature,
      channel
    })

    assert(signedChannel.channel.balance.eq(channelData.balance), 'wrong balance')
    assert(new BN(signedChannel.channel.status).eq(new BN(channelData.status)), 'wrong status')
  })

  it('should create new signedChannel using array', async function() {
    const channelData = await generateChannelData()

    const channel = new Channel(undefined, channelData)
    const signature = await utils.sign(await channel.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedChannelA = new SignedChannel(undefined, {
      signature,
      channel
    })
    const signedChannelB = new SignedChannel({
      bytes: signedChannelA.buffer,
      offset: signedChannelA.byteOffset
    })

    assert(signedChannelB.channel.balance.eq(channelData.balance), 'wrong balance')
    assert(new BN(signedChannelB.channel.status).eq(new BN(channelData.status)), 'wrong status')
  })

  it('should verify signedChannel', async function() {
    const channelData = await generateChannelData()
    const channel = new Channel(undefined, channelData)

    const signature = await utils.sign(await channel.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedChannel = new SignedChannel(undefined, {
      signature,
      channel
    })

    const signer = new Hash(await signedChannel.signer)
    const userAPubKey = await utils.privKeyToPubKey(userA)

    assert(signer.eq(userAPubKey), 'signer incorrect')
  })
})
