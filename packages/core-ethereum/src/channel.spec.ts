import type Indexer from './indexer'
import type { ChainWrapper } from './ethereum'
import {
  ChannelEntry,
  Hash,
  PublicKey,
  Challenge,
  Balance,
  UINT256,
  HoprDB,
  createPoRValuesForSender
} from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import { utils } from 'ethers'
import { Channel } from './channel'
import * as fixtures from './fixtures'

const createChallenge = (secret1: Uint8Array, secret2: Uint8Array): Challenge => {
  return createPoRValuesForSender(secret1, secret2).ticketChallenge
}

const createChainMock = (_channelEntry: ChannelEntry): ChainWrapper => {
  return ({
    async setCommitment() {},
    async getBalance() {},
    async fundChannel() {},
    async openChannel() {},
    async initiateChannelClosure() {},
    async finalizeChannelClosure() {},
    async redeemTicket() {}
  } as unknown) as ChainWrapper
}

const createIndexerMock = (channelEntry: ChannelEntry): Indexer => {
  return {
    async getChannel(_id: Hash) {
      return channelEntry
    }
  } as Indexer
}

const createMocks = () => {
  const selfPrivateKey = utils.arrayify(fixtures.ACCOUNT_A.privateKey)
  const self = PublicKey.fromPrivKey(selfPrivateKey)
  const counterparty = PublicKey.fromPrivKey(utils.arrayify(fixtures.ACCOUNT_B.privateKey))
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
    'CLOSED',
    new UINT256(new BN(1)),
    new UINT256(new BN(0)),
    false
  )
  const challange = createChallenge(
    Hash.create(new Uint8Array([1])).serialize(),
    Hash.create(new Uint8Array([2])).serialize()
  )

  const indexer = createIndexerMock(channelEntry)
  const chain = createChainMock(channelEntry)

  return {
    self,
    privateKey: selfPrivateKey,
    counterparty,
    db,
    indexer,
    chain,
    channelEntry,
    challange,
    nextCommitmentPartyA,
    nextCommitmentPartyB
  }
}

describe('test channel', function () {
  let mocks: ReturnType<typeof createMocks>
  let channel: Channel

  beforeEach(function () {
    mocks = createMocks()
    channel = new Channel(mocks.self, mocks.counterparty, mocks.db, mocks.chain, mocks.indexer, mocks.privateKey)
  })

  it('should create channel', async function () {
    assert.strictEqual(channel.getId().toHex(), fixtures.CHANNEL_ID)
    assert.strictEqual(
      Channel.generateId(mocks.self.toAddress(), mocks.counterparty.toAddress()).toHex(),
      fixtures.CHANNEL_ID
    )
    assert.strictEqual(
      utils.hexlify((await channel.getState()).serialize()),
      utils.hexlify(mocks.channelEntry.serialize())
    )
  })
})
