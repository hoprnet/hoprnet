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
  createPoRValuesForSender,
  ChannelStatus,
  generateChannelId
} from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import { utils } from 'ethers'
import * as fixtures from './fixtures'

const createChallenge = (secret1: Uint8Array, secret2: Uint8Array): Challenge => {
  return createPoRValuesForSender(secret1, secret2).ticketChallenge
}

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

const createIndexerMock = (channelUsThem: ChannelEntry, channelThemUs: ChannelEntry): Indexer => {
  return {
    async getChannel(id: Hash) {
      return id.eq(channelUsThem.getId()) ? channelUsThem : channelThemUs
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

  const channelUsThem = new ChannelEntry(
    self.toAddress(),
    counterparty.toAddress(),
    new Balance(new BN(7)),
    commitmentPartyA,
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(1)),
    new UINT256(new BN(0))
  )
  const channelThemUs = new ChannelEntry(
    counterparty.toAddress(),
    self.toAddress(),
    new Balance(new BN(3)),
    commitmentPartyB,
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(0)),
    new UINT256(new BN(0))
  )
  const challange = createChallenge(
    Hash.create(new Uint8Array([1])).serialize(),
    Hash.create(new Uint8Array([2])).serialize()
  )

  const indexer = createIndexerMock(channelUsThem, channelThemUs)
  const chain = createChainMock()

  return {
    self,
    privateKey: selfPrivateKey,
    counterparty,
    db,
    indexer,
    chain,
    channelUsThem,
    channelThemUs,
    challange,
    nextCommitmentPartyA,
    nextCommitmentPartyB
  }
}

describe('test channel', function () {
  let mocks: ReturnType<typeof createMocks>

  beforeEach(function () {
    mocks = createMocks()
  })

  it('should create channel', async function () {
    assert.strictEqual(
      generateChannelId(mocks.self.toAddress(), mocks.counterparty.toAddress()).toHex(),
      fixtures.CHANNEL_ID
    )
  })
})
