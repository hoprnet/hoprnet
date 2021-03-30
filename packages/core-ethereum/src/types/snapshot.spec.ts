import { expect } from 'chai'
import BN from 'bn.js'
import Snapshot from './snapshot'

describe('Snapshot', function () {
  it('should be empty', function () {
    const snap = Snapshot.deserialize(new Snapshot(new BN(0), new BN(0), new BN(0)).serialize())

    expect(snap.blockNumber.toString()).to.equal('0')
    expect(snap.transactionIndex.toString()).to.equal('0')
    expect(snap.logIndex.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const snap = Snapshot.deserialize(new Snapshot(new BN(1), new BN(2), new BN(3)).serialize())

    expect(snap.blockNumber.toString()).to.equal('1')
    expect(snap.transactionIndex.toString()).to.equal('2')
    expect(snap.logIndex.toString()).to.equal('3')
  })
})
