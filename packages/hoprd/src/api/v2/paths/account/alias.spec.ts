import { _createTestMocks, STATUS_CODES } from '../../'
import assert from 'assert'
import { setAlias, removeAlias, getAlias } from './alias'

const peerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
const invalidPeerId = 'definetly not a valid peerId'
const alias1 = 'alias1'

describe('setAlias', function () {
  const mocks = _createTestMocks()

  it('should set alias successfuly', function () {
    setAlias(mocks, alias1, peerId)
    assert.equal(mocks.getState().aliases.size, 1)
    assert.equal(mocks.getState().aliases.get('alias1').toB58String(), peerId)
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(() => setAlias(mocks, alias1, invalidPeerId), STATUS_CODES.INVALID_PEERID)
  })
})

describe('removeAlias', function () {
  const mocks = _createTestMocks()

  it('should remove alias successfuly', function () {
    setAlias(mocks, alias1, peerId)
    removeAlias(mocks, alias1)
    assert.equal(mocks.getState().aliases.size, 0)
    assert.equal(mocks.getState().aliases.get('alias1').toB58String(), undefined)
  })
})

describe('getAlias', () => {
  const mocks = _createTestMocks()
  setAlias(mocks, alias1, peerId)

  it('should successfuly get alias', () => {
    const alias = getAlias(mocks.getState(), alias1)
    assert.equal(alias, alias1)
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(() => getAlias(mocks.getState(), 'alias2'), STATUS_CODES.PEERID_NOT_FOUND)
  })
})
