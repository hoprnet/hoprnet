import { _createTestState } from '.'
import assert from 'assert'
import { getAlias, setAlias } from './alias'

const peerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
const invalidPeerId = 'definetly not a valid peerId'
const alias1 = 'alias1'

describe('setAlias', () => {
  it('should set alias successfuly', () => {
    const state = _createTestState()
    setAlias({ alias: alias1, peerId, state })
    assert.equal(state.aliases.size, 1)
    assert.equal(state.aliases[0], alias1)
  })
  it('should return error on invalid peerId', () => {
    const state = _createTestState()
    const err = setAlias({ alias: alias1, peerId, state })
    assert.equal(err.message, 'invalidPeerId')
  })
})

describe('getAlias', () => {
  it('should successfuly get aliases', () => {
    const state = _createTestState()
    setAlias({ alias: alias1, peerId, state })
    const aliases = getAlias({ peerId, state }) as string[]
    assert.equal(aliases.length, 1)
    assert.equal(aliases[0], alias1)
  })
  it('should return error on invalid peerId', () => {
    const state = _createTestState()
    const err = getAlias({ peerId, state }) as Error
    assert.equal(err.message, 'invalidPeerId')
  })
  it('should return error when no alias found', () => {
    const state = _createTestState()
    const err = getAlias({ peerId: invalidPeerId, state }) as Error
    assert.equal(err.message, 'aliasNotFound')
  })
})
