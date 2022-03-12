import type { State } from '../../types'
import { Hash } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { setupRestApi } from '../v2'
import express from 'express'
import { Multiaddr } from 'multiaddr'

export const testPeerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
export const testPeerIdInstance = PeerId.createFromB58String(testPeerId)
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

export const ALICE_PEER_ID = PeerId.createFromB58String('16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz')
export const ALICE_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.196/tcp/9091/p2p/${ALICE_PEER_ID.toB58String()}`)
export const BOB_PEER_ID = PeerId.createFromB58String('16Uiu2HAm29vyHEGNm6ebghEs1tm92UxfaNtj8Rc1Y4qV4TAN5xyQ')
export const BOB_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.197/tcp/9091/p2p/${BOB_PEER_ID.toB58String()}`)

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
