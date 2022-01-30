import { _createTestState } from '../../'
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
    assert.equal(state.aliases.get('alias1').toB58String(), peerId)
  })

  it('should throw error on invalid peerId', () => {
    const state = _createTestState()
    assert.throws(() => setAlias({ alias: alias1, peerId: invalidPeerId, state }), /invalidPeerId/)
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

  it('should throw error on invalid peerId', () => {
    const state = _createTestState()
    assert.throws(() => getAlias({ peerId, state }), 'invalidPeerId')
  })

  it('should throw error when no alias found', () => {
    const state = _createTestState()
    assert.throws(() => getAlias({ peerId: invalidPeerId, state }), 'aliasNotFound')
  })
})
