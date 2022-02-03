import { STATUS_CODES } from '../../'
import assert from 'assert'
import { removeAlias, getAlias } from './{alias}'
import { createTestMocks, testPeerId, testAlias } from '../../fixtures'
import { setAlias } from '.'

describe('removeAlias', function () {
  const mocks = createTestMocks()

  it('should remove alias successfuly', function () {
    setAlias(mocks, testAlias, testPeerId)
    removeAlias(mocks, testAlias)
    assert.equal(mocks.getState().aliases.size, 0)
    assert.equal(mocks.getState().aliases.get('alias'), undefined)
  })
})

describe('getAlias', () => {
  const mocks = createTestMocks()
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
