import { _createTestMocks } from '../../'
import assert from 'assert'
import { setAlias } from './alias'
import { getAliases } from './aliases'

const peerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
const alias1 = 'alias1'

describe('getAliases', () => {
  const mocks = _createTestMocks()
  setAlias(mocks, alias1, peerId)

  it('should successfuly get aliases', () => {
    const aliases = getAliases(mocks.getState())
    assert.deepEqual(aliases, { alias1: peerId })
  })
})
