import { expect } from 'chai'
import BN from 'bn.js'
import { snapshotComparator, isConfirmedBlock } from './utils'

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

  it('should be false when blockNumber=5 onChainBlockNumber=0 MaxConf=0', function () {
    expect(isConfirmedBlock(5, 0, 0)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=5 MaxConf=0', function () {
    expect(isConfirmedBlock(5, 5, 0)).to.be.true
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 MaxConf=0', function () {
    expect(isConfirmedBlock(5, 10, 0)).to.be.true
  })

  it('should be false when blockNumber=5 onChainBlockNumber=0 MaxConf=5', function () {
    expect(isConfirmedBlock(5, 0, 5)).to.be.false
  })

  it('should be false when blockNumber=5 onChainBlockNumber=5 MaxConf=5', function () {
    expect(isConfirmedBlock(5, 5, 5)).to.be.false
  })

  it('should be true when blockNumber=5 onChainBlockNumber=10 MaxConf=5', function () {
    expect(isConfirmedBlock(5, 10, 5)).to.be.true
  })
})
