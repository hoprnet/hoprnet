import { getSetting, setSetting, Setting } from '../../logic/settings'
import sinon from 'sinon'
import assert from 'assert'
import { _createTestState } from '../..'

let node = sinon.fake() as any

describe('getSetting', () => {
  it('should return all settings if no settingName provided', () => {
    const state = _createTestState()
    const allSettings = Array.from(Object.entries(state.settings)).map(([name, value]) => ({ name, value }))
    const settings = getSetting({ node, state }) as any[]
    assert.deepEqual(settings, allSettings)
  })

  it('should return value of specific setting when settingName provided', () => {
    const state = _createTestState()
    const setting = getSetting({ node, state, settingName: 'includeRecipient' }) as Setting
    assert.equal(setting.value, state.settings.includeRecipient)
  })

  it('should return error when invalid settingName provided', () => {
    const state = _createTestState()
    const err = getSetting({ node, state, settingName: 'abcd' as any }) as Error
    assert.equal(err.message, 'invalidSettingName')
  })
})

describe('setSetting', () => {
  it('should set setting successfuly', () => {
    const state = _createTestState()
    const err = setSetting({ settingName: 'includeRecipient', value: true, node, state })
    assert.equal(state.settings.includeRecipient, true)
    assert.equal(err, undefined)
  })
  it('should return error when invalid settingName provided', () => {
    const state = _createTestState()
    const err = setSetting({ settingName: 'abcd' as any, value: true, node, state })
    assert.equal(err.message, 'invalidSettingName')
  })
  // NOTE: add case for every setting individually
  it('should return error when invalid value provided ', () => {
    const state = _createTestState()

    const err = setSetting({ settingName: 'includeRecipient', value: 'true', node, state })
    assert.equal(err.message, 'invalidValue')
    const err2 = setSetting({ settingName: 'strategy', value: 'abcd', node, state })
    assert.equal(err2.message, 'invalidValue')
  })
})
