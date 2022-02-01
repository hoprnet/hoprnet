import { _createTestMocks } from '../../'
import assert from 'assert'
import { setAlias } from './alias'
import { getAliases } from './aliases'
import { testAlias, testPeerId } from '../../fixtures'

describe('getAliases', () => {
  const mocks = _createTestMocks()
  setAlias(mocks, testAlias, testPeerId)

  it('should successfuly get aliases', () => {
    const aliases = getAliases(mocks.getState())
    assert.deepEqual(aliases, { alias: testPeerId })
  })
})
