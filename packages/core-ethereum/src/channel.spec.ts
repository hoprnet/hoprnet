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
  generateChannelId,
  HalfKey,
  PRICE_PER_PACKET
} from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import { utils } from 'ethers'
import { Channel, _redeemTicket } from './channel'
import * as fixtures from './fixtures'
import { EventEmitter } from 'events'
import { IndexerEvents } from './indexer/types'

const createChainMock = (): ChainWrapper => {
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

const createIndexerMock = (): Indexer => {
  return {
    async resolvePendingTransaction(_eventType: IndexerEvents, tx: string) {
      return tx
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

  const channelUsThem = new ChannelEntry(
    self,
    counterparty,
    new Balance(new BN(7).mul(PRICE_PER_PACKET)),
    commitmentPartyA,
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(1)),
    new UINT256(new BN(0))
  )
  const channelThemUs = new ChannelEntry(
    counterparty,
    self,
    new Balance(new BN(3).mul(PRICE_PER_PACKET)),
    commitmentPartyB,
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(0)),
    new UINT256(new BN(0))
  )

  const secret1 = new HalfKey(Hash.create(new Uint8Array([1])).serialize())
  const secret2 = new HalfKey(Hash.create(new Uint8Array([2])).serialize())
  const response = Response.fromHalfKeys(secret1, secret2)

  const indexer = createIndexerMock()
  const chain = createChainMock()
  const ev = new EventEmitter()
  const channel = new Channel(self, counterparty, db, chain, indexer, selfPrivateKey, ev)

  return {
    self,
    privateKey: selfPrivateKey,
    counterparty,
    db,
    indexer,
    chain,
    channelUsThem,
    channelThemUs,
    response,
    nextCommitmentPartyA,
    nextCommitmentPartyB,
    secret1,
    secret2,
    channel,
    events: ev
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
    assert.strictEqual((await aliceMocks.channel.usToThem()).getId().toHex(), fixtures.CHANNEL_ID)
    assert.strictEqual(
      generateChannelId(aliceMocks.self.toAddress(), aliceMocks.counterparty.toAddress()).toHex(),
      fixtures.CHANNEL_ID
    )
    assert.strictEqual(
      utils.hexlify((await aliceMocks.channel.usToThem()).serialize()),
      utils.hexlify(aliceMocks.channelUsThem.serialize())
    )
  })

  it("should validate ticket's response", async function () {
    const ticket = await aliceMocks.channel.createTicket(2, aliceMocks.response.toChallenge())

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

    const goodResponse = await _redeemTicket(
      aliceMocks.self,
      goodAck,
      bobMocks.db,
      bobMocks.chain,
      bobMocks.indexer,
      bobMocks.events
    )
    assert(goodResponse.status === 'SUCCESS')

    const badResponse = await _redeemTicket(
      aliceMocks.self,
      badAck,
      bobMocks.db,
      bobMocks.chain,
      bobMocks.indexer,
      bobMocks.events
    )
    assert(badResponse.status === 'FAILURE' && badResponse.message === 'Invalid response to acknowledgement')

    const aBalances = await aliceMocks.channel.balanceToThem()
    assert(aBalances.minimum.eq(aBalances.maximum.sub(PRICE_PER_PACKET)), 'max and min balance diverge')
  })

  it("should validate ticket's preimage", async function () {
    const ticket = await aliceMocks.channel.createTicket(2, aliceMocks.response.toChallenge())

    const acknowledgement = new AcknowledgedTicket(
      ticket,
      aliceMocks.response,
      new Hash(new Uint8Array({ length: Hash.SIZE })), // empty preimage
      aliceMocks.self
    )

    const response = await _redeemTicket(
      aliceMocks.self,
      acknowledgement,
      bobMocks.db,
      bobMocks.chain,
      bobMocks.indexer,
      bobMocks.events
    )
    assert(response.status === 'FAILURE' && response.message === 'PreImage is empty.')
  })
})
