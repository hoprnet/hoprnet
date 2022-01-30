import type { State } from '../../types'

/**
 * Used in tests to create state mocking.
 * @returns testing mocks
 */
export const _createTestMocks = () => {
  let state: State = {
    aliases: new Map(),
    settings: { includeRecipient: false, strategy: 'passive' }
  }

  return {
    setState(s: State) {
      state = s
    },
    getState() {
      return state
    }
  }
}

export const STATUS_CODES = {
  SUCCESS: 'SUCCESS',
  UNKNOWN_FAILURE: 'UNKNOWN_FAILURE',
  INVALID_PEERID: 'INVALID_PEERID',
  PEERID_NOT_FOUND: 'PEERID_NOT_FOUND',
  TIMEOUT: 'TIMEOUT',
  INVALID_SETTING: 'INVALID_SETTING',
  INVALID_SETTING_VALUE: 'INVALID_SETTING_VALUE'
}
