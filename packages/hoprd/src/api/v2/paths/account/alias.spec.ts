import { _createTestMocks, STATUS_CODES } from '../../'
import assert from 'assert'
import { setAlias, removeAlias, getAlias } from './alias'
import { invalidTestPeerId, testPeerId, testAlias } from '../../fixtures'

describe('setAlias', function () {
  const mocks = _createTestMocks()

  it('should set alias successfuly', function () {
    setAlias(mocks, testAlias, testPeerId)
    assert.equal(mocks.getState().aliases.size, 1)
    assert.equal(mocks.getState().aliases.get('alias').toB58String(), testPeerId)
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(
      () => setAlias(mocks, testAlias, invalidTestPeerId),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
  })
})

describe('removeAlias', function () {
  const mocks = _createTestMocks()

  it('should remove alias successfuly', function () {
    setAlias(mocks, testAlias, testPeerId)
    removeAlias(mocks, testAlias)
    assert.equal(mocks.getState().aliases.size, 0)
    assert.equal(mocks.getState().aliases.get('alias'), undefined)
  })
})

describe('getAlias', () => {
  const mocks = _createTestMocks()
  setAlias(mocks, testAlias, testPeerId)

  it('should successfuly get alias', () => {
    const alias = getAlias(mocks.getState(), testAlias)
    assert.equal(alias, testPeerId)
  })

  it('should throw error on invalid peerId', () => {
    assert.throws(
      () => getAlias(mocks.getState(), 'alias2'),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.PEERID_NOT_FOUND)
      }
    )
  })
})
