import { expect } from 'chai'
import BN from 'bn.js'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Public } from '../types'
import * as utils from './utils'
import * as fixtures from './utils.fixtures.spec'

describe('test snapshotComparator', function () {
  const EVENT_1_0_0 = {
    blockNumber: new BN(1),
    transactionIndex: new BN(0),
    logIndex: new BN(0)
  }
  const EVENT_1_1_0 = {
    blockNumber: new BN(1),
    transactionIndex: new BN(1),
    logIndex: new BN(0)
  }
  const EVENT_1_1_1 = {
    blockNumber: new BN(1),
    transactionIndex: new BN(1),
    logIndex: new BN(1)
  }
  const EVENT_2_0_0 = {
    blockNumber: new BN(2),
    transactionIndex: new BN(0),
    logIndex: new BN(0)
  }

  it('should return zero when event is the same', function () {
    expect(utils.snapshotComparator(EVENT_1_0_0, EVENT_1_0_0)).to.equal(0)
  })

  it('should return negative number when block number is older', function () {
    expect(utils.snapshotComparator(EVENT_1_0_0, EVENT_2_0_0)).to.below(0)
  })

  it('should return negative number when transaction index is older', function () {
    expect(utils.snapshotComparator(EVENT_1_0_0, EVENT_1_1_0)).to.below(0)
  })

  it('should return negative number when log index is older', function () {
    expect(utils.snapshotComparator(EVENT_1_0_0, EVENT_1_1_1)).to.below(0)
  })

  it('should return positive number when block number is younger', function () {
    expect(utils.snapshotComparator(EVENT_2_0_0, EVENT_1_0_0)).to.above(0)
  })

  it('should return positive number when transaction index is younger', function () {
    expect(utils.snapshotComparator(EVENT_2_0_0, EVENT_1_1_0)).to.above(0)
  })

  it('should return positive number when log index is younger', function () {
    expect(utils.snapshotComparator(EVENT_2_0_0, EVENT_1_1_1)).to.above(0)
  })
})

describe('test isConfirmedBlock', function () {
  it('should be true when everything is 0', function () {
    expect(utils.isConfirmedBlock(0, 0, 0)).to.be.true
  })

  it('should be false when blockNumber=5 onChainBlockNumber=0 maxConf=0', function () {
    expect(utils.isConfirmedBlock(5, 0, 0)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=5 maxConf=0', function () {
    expect(utils.isConfirmedBlock(5, 5, 0)).to.be.true
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 maxConf=0', function () {
    expect(utils.isConfirmedBlock(5, 10, 0)).to.be.true
  })

  it('should be false when blockNumber=5 onChainBlockNumber=0 maxConf=5', function () {
    expect(utils.isConfirmedBlock(5, 0, 5)).to.be.false
  })

  it('should be false when blockNumber=5 onChainBlockNumber=5 maxConf=5', function () {
    expect(utils.isConfirmedBlock(5, 5, 5)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 maxConf=5', function () {
    expect(utils.isConfirmedBlock(5, 10, 5)).to.be.true
  })
})

describe('test isSyncing', function () {
  it('should be syncing', function () {
    expect(utils.isSyncing(100, 100)).to.be.true
    expect(utils.isSyncing(100, 96)).to.be.true
  })

  it('should not be syncing', function () {
    expect(utils.isSyncing(100, 95)).to.be.false
  })
})

describe('test indexer storage', function () {
  const partyA = new Public(stringToU8a('0x03767782fdb4564f0a2dee849d9fc356207dd89f195fcfd69ce0b02c6f03dfda40'))
  const partyB = new Public(stringToU8a('0x024890561acbe7d1b8832621488a887291eedec2b4bc07a464fef7a9b4c7857cf8'))
  const db = new LevelUp(Memdown())

  beforeEach(async function () {
    await db.clear()
  })

  it('should store latest block number', async function () {
    await utils.updateLatestBlockNumber(db, new BN(1))

    const latestBlockNumber = await utils.getLatestBlockNumber(db)
    expect(latestBlockNumber.toString()).to.equal('1')
  })

  it('should store channelEntries', async function () {
    await utils.updateChannelEntry(db, partyA, partyB, fixtures.CHANNEL_ENTRY)

    const partyAChannelEntries = await utils.getChannelEntries(db, partyA)
    expect(partyAChannelEntries).length(1)
    expect(partyAChannelEntries[0].partyA).to.deep.equal(partyA)
    expect(partyAChannelEntries[0].partyB).to.deep.equal(partyB)
    expect(partyAChannelEntries[0].channelEntry).to.deep.equal(fixtures.CHANNEL_ENTRY)

    const partyBChannelEntries = await utils.getChannelEntries(db, partyB)
    expect(partyBChannelEntries).length(1)
    expect(partyBChannelEntries[0].partyA).to.deep.equal(partyA)
    expect(partyBChannelEntries[0].partyB).to.deep.equal(partyB)
    expect(partyBChannelEntries[0].channelEntry).to.deep.equal(fixtures.CHANNEL_ENTRY)

    const channelEntry = await utils.getChannelEntry(db, partyA, partyB)
    expect(channelEntry).to.deep.equal(fixtures.CHANNEL_ENTRY)
  })

  it('should store latest confirmed snapshot', async function () {
    await utils.updateChannelEntry(db, partyA, partyB, fixtures.CHANNEL_ENTRY)

    const snapshot = await utils.getLatestConfirmedSnapshot(db)
    expect(snapshot.blockNumber.toString()).to.equal(fixtures.CHANNEL_ENTRY.blockNumber.toString())
    expect(snapshot.transactionIndex.toString()).to.equal(fixtures.CHANNEL_ENTRY.transactionIndex.toString())
    expect(snapshot.logIndex.toString()).to.equal(fixtures.CHANNEL_ENTRY.logIndex.toString())
  })

  it('should store account entry', async function () {
    const accountId = await partyA.toAccountId()

    await utils.updateAccountEntry(db, accountId, fixtures.ACCOUNT_ENTRY)

    const accountEntry = await utils.getAccountEntry(db, accountId)
    expect(accountEntry).to.deep.equal(fixtures.ACCOUNT_ENTRY)
  })
})
