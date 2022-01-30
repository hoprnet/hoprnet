import sinon from 'sinon'
import assert from 'assert'
import { STATUS_CODES, _createTestMocks } from '../../'
import { getSetting, setSetting, Setting } from './settings'

let node = sinon.fake() as any

describe('getSetting', () => {
  it('should return all settings if no settingName provided', () => {
    const stateOps = _createTestMocks()
    const state = stateOps.getState()
    const allSettings = Array.from(Object.entries(state.settings)).map(([name, value]) => ({ name, value }))
    const settings = getSetting({ node, state })
    assert.deepEqual(settings, allSettings)
  })

  it('should return value of specific setting when settingName provided', () => {
    const stateOps = _createTestMocks()
    const state = stateOps.getState()
    const setting = getSetting({ node, state, settingName: 'includeRecipient' }) as Setting
    assert.equal(setting.value, state.settings.includeRecipient)
  })

  it('should return error when invalid settingName provided', () => {
    const stateOps = _createTestMocks()
    const state = stateOps.getState()
    const err = getSetting({ node, state, settingName: 'abcd' as any }) as any
    assert.equal(err.message, STATUS_CODES.INVALID_SETTING)
  })
})

describe('setSetting', () => {
  it('should set setting successfuly', () => {
    const stateOps = _createTestMocks()
    const err = setSetting({ settingName: 'includeRecipient', value: true, node, stateOps })
    const state = stateOps.getState()
    assert.equal(state.settings.includeRecipient, true)
    assert.equal(err, undefined)
  })
  it('should return error when invalid settingName provided', () => {
    const stateOps = _createTestMocks()
    assert.throws(
      () => setSetting({ settingName: 'abcd' as any, value: true, node, stateOps }),
      STATUS_CODES.INVALID_SETTING
    )
  })
  // NOTE: add case for every setting individually
  it('should return error when invalid value provided ', () => {
    const stateOps = _createTestMocks()
    const state = stateOps.getState()

    assert.throws(
      () => setSetting({ settingName: 'includeRecipient', value: 'true', node, stateOps }),
      STATUS_CODES.INVALID_SETTING_VALUE
    )
    assert.throws(
      () => setSetting({ settingName: 'strategy', value: 'abcd', node, stateOps }),
      STATUS_CODES.INVALID_SETTING_VALUE
    )
  })
})
