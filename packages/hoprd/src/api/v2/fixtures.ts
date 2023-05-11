import type { State } from '../../types.js'
import express from 'express'
import { peerIdFromString } from '@libp2p/peer-id'
import { Multiaddr } from '@multiformats/multiaddr'
import { setupRestApi } from '../v2.js'
import {
  Balance,
  BalanceType,
  Hash,
  U256,
  ChannelEntry,
  HoprDB,
  PublicKey,
  stringToU8a,
  ChannelStatus
} from '@hoprnet/hopr-utils'

export const ALICE_PEER_ID = peerIdFromString('16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz')
export const ALICE_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.196/tcp/9091/p2p/${ALICE_PEER_ID.toString()}`)
export const ALICE_NATIVE_ADDR = () => PublicKey.from_peerid_str(ALICE_PEER_ID.toString()).to_address()
export const BOB_PEER_ID = peerIdFromString('16Uiu2HAm29vyHEGNm6ebghEs1tm92UxfaNtj8Rc1Y4qV4TAN5xyQ')
export const BOB_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.197/tcp/9091/p2p/${BOB_PEER_ID.toString()}`)
export const CHARLIE_PEER_ID = peerIdFromString('16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12')
export const INVALID_PEER_ID = 'definetly not a valid peerId'

export const TICKET_MOCK = {
  counterparty: { to_hex: () => '', to_string: () => '' },
  challenge: { to_hex: () => '', to_string: () => '' },
  epoch: { to_hex: () => '', to_string: () => '' },
  index: { to_hex: () => '', to_string: () => '' },
  amount: { to_hex: () => '', to_string: () => '' },
  win_prob: { to_hex: () => '', to_string: () => '' },
  channel_epoch: { to_hex: () => '', to_string: () => '' },
  signature: { to_hex: () => '', to_string: () => '' }
}

export function channelEntryCreateMock(): ChannelEntry {
  const pub = PublicKey.from_privkey(stringToU8a('0x1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'))
  return new ChannelEntry(
    pub.clone(),
    pub,
    new Balance('1', BalanceType.HOPR),
    Hash.create([]),
    U256.one(),
    U256.one(),
    ChannelStatus.Closed,
    U256.one(),
    U256.one()
  )
}

/**
 * Creates express app and initializes all routes for testing
 * @returns apiInstance for openAPI validation and express instance for supertest requests
 */
export const createTestApiInstance = async (node: any) => {
  const service = express()
  return {
    api: await setupRestApi(service, '/api/v2', node, createTestMocks(), {
      disableApiAuthentication: true
    }),
    service
  }
}

/**
 * Creates express app and initializes all routes for testing
 * @returns apiInstance for openAPI validation and express instance for supertest requests with superuser authentication token set to superuser
 */
export const createAuthenticatedTestApiInstance = async (node: any) => {
  const service = express()
  return {
    api: await setupRestApi(service, '/api/v2', node, createTestMocks(), {
      apiToken: 'superuser'
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
    settings: { includeRecipient: false, strategy: 'passive', maxAutoChannels: undefined, autoRedeemTickets: false }
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
 * Used in tests to create a db mock for generic object storage
 * @returns testing mocked db
 */
export const createMockDb = () => {
  const store = {}
  const key = (ns: string, id: string) => `${ns}:${id}`
  const db = {
    putSerializedObject: async (ns: string, id: string, object: Uint8Array): Promise<void> => {
      store[key(ns, id)] = object
      return
    },
    getSerializedObject: async (ns: string, id: string): Promise<Uint8Array | undefined> => {
      return store[key(ns, id)]
    },
    deleteObject: async (ns: string, id: string): Promise<void> => {
      delete store[key(ns, id)]
      return
    }
  }
  return db as any as HoprDB
}
