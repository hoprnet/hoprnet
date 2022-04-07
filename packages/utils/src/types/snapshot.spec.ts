import assert from 'assert'
import BN from 'bn.js'
import { Snapshot } from './snapshot'

describe('Snapshot', function () {
  it('should be empty', function () {
    const snap = Snapshot.deserialize(new Snapshot(new BN(0), new BN(0), new BN(0)).serialize())

    assert(snap.blockNumber.toString() === '0')
    assert(snap.transactionIndex.toString() === '0')
    assert(snap.logIndex.toString() === '0')
  })

  it('should contain the right values', function () {
    const snap = Snapshot.deserialize(new Snapshot(new BN(1), new BN(2), new BN(3)).serialize())

    assert(snap.blockNumber.toString() === '1')
    assert(snap.transactionIndex.toString() === '2')
    assert(snap.logIndex.toString() === '3')
  })
})
