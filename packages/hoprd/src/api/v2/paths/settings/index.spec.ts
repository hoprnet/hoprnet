import sinon from 'sinon'
import assert from 'assert'
import { createTestMocks } from '../../fixtures'
import { getSettings } from '.'

let node = sinon.fake() as any
node.getChannelStrategy = sinon.fake.returns('passive')

describe('getSetting', () => {
  it('should return all settings', () => {
    const stateOps = createTestMocks()
    const state = stateOps.getState()
    const settings = getSettings(state)
    assert.deepEqual(settings, state.settings)
  })
})
