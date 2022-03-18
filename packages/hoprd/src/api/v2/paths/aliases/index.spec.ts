import { STATUS_CODES } from '../../utils'
import assert from 'assert'
import { getAliases, setAlias } from '.'
import { createTestMocks, invalidTestPeerId, testAlias, testPeerId } from '../../fixtures'

describe('getAliases', () => {
  const mocks = createTestMocks()
  setAlias(mocks, testAlias, testPeerId)

  it('should successfuly get aliases', () => {
    const aliases = getAliases(mocks.getState())
    assert.deepEqual(aliases, { alias: testPeerId })
  })
})

describe('setAlias', function () {
  const mocks = createTestMocks()

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
