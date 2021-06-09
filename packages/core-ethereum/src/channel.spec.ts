import type Indexer from './indexer'
import type { ChainWrapper } from './ethereum'
import {
  ChannelEntry,
  Hash,
  PublicKey,
  Balance,
  UINT256,
  HoprDB,
  ChannelStatus,
  AcknowledgedTicket,
  Response,
  HalfKey
} from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import { utils } from 'ethers'
import { Channel } from './channel'
import * as fixtures from './fixtures'

const createChainMock = (_channelEntry: ChannelEntry): ChainWrapper => {
  return {
    async setCommitment() {},
    async getBalance() {},
    async fundChannel() {},
    async openChannel() {},
    async initiateChannelClosure() {},
    async finalizeChannelClosure() {},
    async redeemTicket() {}
  } as unknown as ChainWrapper
}

const createIndexerMock = (channelEntry: ChannelEntry): Indexer => {
  return {
    async getChannel(_id: Hash) {
      return channelEntry
    }
  } as Indexer
}

const createMocks = (from: string, to: string) => {
  const selfPrivateKey = utils.arrayify(from)
  const self = PublicKey.fromPrivKey(selfPrivateKey)
  const counterparty = PublicKey.fromPrivKey(utils.arrayify(to))
  const db = HoprDB.createMock()

  const nextCommitmentPartyA = Hash.create(new Uint8Array([0]))
  const commitmentPartyA = nextCommitmentPartyA.hash()
  const nextCommitmentPartyB = Hash.create(new Uint8Array([1]))
  const commitmentPartyB = nextCommitmentPartyB.hash()

  const channelEntry = new ChannelEntry(
    self.toAddress(),
    counterparty.toAddress(),
    new Balance(new BN(7)),
    new Balance(new BN(3)),
    commitmentPartyA,
    commitmentPartyB,
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(1)),
    new UINT256(new BN(0)),
    false
  )

  const secret1 = new HalfKey(Hash.create(new Uint8Array([1])).serialize())
  const secret2 = new HalfKey(Hash.create(new Uint8Array([2])).serialize())
  const response = Response.fromHalfKeys(secret1, secret2)

  const indexer = createIndexerMock(channelEntry)
  const chain = createChainMock(channelEntry)
  const channel = new Channel(self, counterparty, db, chain, indexer, selfPrivateKey)

  return {
    self,
    privateKey: selfPrivateKey,
    counterparty,
    db,
    indexer,
    chain,
    channelEntry,
    response,
    nextCommitmentPartyA,
    nextCommitmentPartyB,
    secret1,
    secret2,
    channel
  }
}

describe('test channel', function () {
  const alicePrivKey = fixtures.ACCOUNT_A.privateKey
  const bobPrivKey = fixtures.ACCOUNT_B.privateKey

  let aliceMocks: ReturnType<typeof createMocks>
  let bobMocks: ReturnType<typeof createMocks>

  beforeEach(function () {
    aliceMocks = createMocks(alicePrivKey, bobPrivKey)
    bobMocks = createMocks(bobPrivKey, alicePrivKey)
  })

  it('should create channel', async function () {
    assert.strictEqual(aliceMocks.channel.getId().toHex(), fixtures.CHANNEL_ID)
    assert.strictEqual(
      Channel.generateId(aliceMocks.self.toAddress(), aliceMocks.counterparty.toAddress()).toHex(),
      fixtures.CHANNEL_ID
    )
    assert.strictEqual(
      utils.hexlify((await aliceMocks.channel.getState()).serialize()),
      utils.hexlify(aliceMocks.channelEntry.serialize())
    )
  })

  it("should validate ticket's response", async function () {
    const ticket = await aliceMocks.channel.createTicket(
      new Balance(new BN(1)),
      aliceMocks.response.toChallenge(),
      new BN(1)
    )

    const goodAck = new AcknowledgedTicket(
      ticket,
      aliceMocks.response,
      aliceMocks.nextCommitmentPartyA,
      aliceMocks.self
    )

    const badAck = new AcknowledgedTicket(
      ticket,
      Response.fromHalfKeys(aliceMocks.secret1, aliceMocks.secret1), // incorrect response
      aliceMocks.nextCommitmentPartyA,
      aliceMocks.self
    )

    const goodResponse = await bobMocks.channel.redeemTicket(goodAck)
    assert(goodResponse.status === 'SUCCESS')

    const badResponse = await bobMocks.channel.redeemTicket(badAck)
    assert(badResponse.status === 'FAILURE' && badResponse.message === 'Invalid response to acknowledgement')
  })

  it("should validate ticket's preimage", async function () {
    const ticket = await aliceMocks.channel.createTicket(
      new Balance(new BN(1)),
      aliceMocks.response.toChallenge(),
      new BN(1)
    )

    const acknowledgement = new AcknowledgedTicket(
      ticket,
      aliceMocks.response,
      new Hash(new Uint8Array({ length: Hash.SIZE })), // empty preimage
      aliceMocks.self
    )

    const response = await bobMocks.channel.redeemTicket(acknowledgement)
    assert(response.status === 'FAILURE' && response.message === 'PreImage is empty.')
  })
})
