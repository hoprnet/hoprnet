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

/**
 * At the moment, using our own custom codes
 * and validations in the possibilty we want to
 * reuse the code for commands, will change if said
 * otherwise.
 */
export const STATUS_CODES = {
  SUCCESS: 'SUCCESS',
  UNKNOWN_FAILURE: 'UNKNOWN_FAILURE',
  PEERID_NOT_FOUND: 'PEERID_NOT_FOUND',
  INVALID_PEERID: 'INVALID_PEERID',
  INVALID_CURRENCY: 'INVALID_CURRENCY',
  INVALID_AMOUNT: 'INVALID_AMOUNT',
  INVALID_ADDRESS: 'INVALID_ADDRESS',
  NOT_ENOUGH_BALANCE: 'NOT_ENOUGH_BALANCE',
  CHANNEL_ALREADY_OPEN: 'CHANNEL_ALREADY_OPEN',
  TIMEOUT: 'TIMEOUT',
  INVALID_SETTING: 'INVALID_SETTING',
  INVALID_SETTING_VALUE: 'INVALID_SETTING_VALUE'
}
