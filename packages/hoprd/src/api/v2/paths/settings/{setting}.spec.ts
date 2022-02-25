import sinon from 'sinon'
import assert from 'assert'
import { STATUS_CODES } from '../../utils'
import { createTestMocks } from '../../fixtures'
import { setSetting } from './{setting}'
import { SettingKey } from '../../../../types'

let node = sinon.fake() as any
node.getChannelStrategy = sinon.fake.returns('passive')

describe('setSetting', () => {
  it('should set setting successfuly', () => {
    const stateOps = createTestMocks()
    setSetting(node, stateOps, SettingKey.INCLUDE_RECIPIENT, true)
    const state = stateOps.getState()
    assert.equal(state.settings.includeRecipient, true)
  })

  it('should return error when invalid setting key is provided', () => {
    const stateOps = createTestMocks()
    assert.throws(
      () => setSetting(node, stateOps, 'invalidKey' as any, true),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_SETTING)
      }
    )
  })

  it('should throw error when invalid value provided ', () => {
    const stateOps = createTestMocks()

    assert.throws(
      () => setSetting(node, stateOps, SettingKey.INCLUDE_RECIPIENT, 'true'),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_SETTING_VALUE)
      }
    )
    assert.throws(
      () => setSetting(node, stateOps, SettingKey.STRATEGY, 'abcd'),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_SETTING_VALUE)
      }
    )
  })
})
