import type { State } from '../../types'
import { Hash } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { setupRestApi } from '../v2'
import express from 'express'

export const testPeerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
export const testPeerIdInstance = PeerId.createFromB58String('16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12')
export const invalidTestPeerId = 'definetly not a valid peerId'
export const testAlias = 'alias'
export const testEthAddress = '0x07c97c4f845b4698D79036239153bB381bc72ad3'
export const testChannelId = new Hash(new Uint8Array(Hash.SIZE))
// temporary mock, had problems with mocking formatTicket function in integration test
export const testTicket = {
  counterparty: { toHex: () => '', toBN: () => '' },
  challenge: { toHex: () => '', toBN: () => '' },
  epoch: { toHex: () => '', toBN: () => '' },
  index: { toHex: () => '', toBN: () => '' },
  amount: { toHex: () => '', toBN: () => '' },
  winProb: { toHex: () => '', toBN: () => '' },
  channelEpoch: { toHex: () => '', toBN: () => '' },
  signature: { toHex: () => '', toBN: () => '' }
}

/**
 * Creates express app and initializes all routes for testing
 * @returns apiInstance for openAPI validation and express instance for supertest requests
 */
export const createTestApiInstance = (node: any) => {
  const service = express()
  return {
    api: setupRestApi(service, '/api/v2', node, createTestMocks(), {
      testNoAuthentication: true
    }),
    service
  }
}

/**
 * Used in tests to create state mocking.
 * @returns testing mocks
 */
export const createTestMocks = () => {
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
