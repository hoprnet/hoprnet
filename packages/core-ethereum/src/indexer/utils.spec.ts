import { expect } from 'chai'
import { snapshotComparator, isConfirmedBlock, isSyncing } from './utils'

describe('test snapshotComparator', function () {
  const EVENT_1_0_0 = {
    blockNumber: 1,
    transactionIndex: 0,
    logIndex: 0
  }
  const EVENT_1_1_0 = {
    blockNumber: 1,
    transactionIndex: 1,
    logIndex: 0
  }
  const EVENT_1_1_1 = {
    blockNumber: 1,
    transactionIndex: 1,
    logIndex: 1
  }
  const EVENT_2_0_0 = {
    blockNumber: 2,
    transactionIndex: 0,
    logIndex: 0
  }

  it('should return zero when event is the same', function () {
    expect(snapshotComparator(EVENT_1_0_0, EVENT_1_0_0)).to.equal(0)
  })

  it('should return negative number when block number is older', function () {
    expect(snapshotComparator(EVENT_1_0_0, EVENT_2_0_0)).to.below(0)
  })

  it('should return negative number when transaction index is older', function () {
    expect(snapshotComparator(EVENT_1_0_0, EVENT_1_1_0)).to.below(0)
  })

  it('should return negative number when log index is older', function () {
    expect(snapshotComparator(EVENT_1_0_0, EVENT_1_1_1)).to.below(0)
  })

  it('should return positive number when block number is younger', function () {
    expect(snapshotComparator(EVENT_2_0_0, EVENT_1_0_0)).to.above(0)
  })

  it('should return positive number when transaction index is younger', function () {
    expect(snapshotComparator(EVENT_2_0_0, EVENT_1_1_0)).to.above(0)
  })

  it('should return positive number when log index is younger', function () {
    expect(snapshotComparator(EVENT_2_0_0, EVENT_1_1_1)).to.above(0)
  })
})

describe('test isConfirmedBlock', function () {
  it('should be true when everything is 0', function () {
    expect(isConfirmedBlock(0, 0, 0)).to.be.true
  })

  it('should be false when blockNumber=5 onChainBlockNumber=0 maxConf=0', function () {
    expect(isConfirmedBlock(5, 0, 0)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=5 maxConf=0', function () {
    expect(isConfirmedBlock(5, 5, 0)).to.be.true
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 maxConf=0', function () {
    expect(isConfirmedBlock(5, 10, 0)).to.be.true
  })

  it('should be false when blockNumber=5 onChainBlockNumber=0 maxConf=5', function () {
    expect(isConfirmedBlock(5, 0, 5)).to.be.false
  })

  it('should be false when blockNumber=5 onChainBlockNumber=5 maxConf=5', function () {
    expect(isConfirmedBlock(5, 5, 5)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 maxConf=5', function () {
    expect(isConfirmedBlock(5, 10, 5)).to.be.true
  })
})

describe('test isSyncing', function () {
  it('should be syncing', function () {
    expect(isSyncing(100, 100)).to.be.true
    expect(isSyncing(100, 96)).to.be.true
  })

  it('should not be syncing', function () {
    expect(isSyncing(100, 95)).to.be.false
  })
})
