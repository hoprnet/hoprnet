import type { State } from '../../types'

export const _createTestState = (): State => ({
  aliases: new Map(),
  settings: { includeRecipient: false, strategy: 'passive' }
})
