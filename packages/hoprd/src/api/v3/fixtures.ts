import express from 'express'
import { peerIdFromString } from '@libp2p/peer-id'
import { Multiaddr } from '@multiformats/multiaddr'

import { setupRestApi } from '../v3.js'
import {
  AccountEntry,
  OffchainPublicKey,
  Address,
  ChannelEntry,
  ChannelStatus,
  Balance,
  BalanceType,
  U256
} from '@hoprnet/hopr-utils'

import { MessageInbox, MessageInboxConfiguration, hoprd_hoprd_initialize_crate } from '../../../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import type { PeerId } from '@libp2p/interface-peer-id'
import type { State } from '../../types.js'

const ALICE_PEER_ID_STR: string = '12D3KooWLYKsvDB4xEELYoHXxeStj2gzaDXjra2uGaFLpKCZkJHs'
const ALICE_ETHEREUM_ADDR_STR: string = '0xd08933750bffb86861d1d76e559382658ef4d761'
const BOB_PEER_ID_STR: string = '12D3KooWRNw2pJC9748Fmq4WNV27HoSTcX3r37132FLkQMrbKAiC'
const BOB_ETHEREUM_ADDR_STR: string = '0xd08933750bffb86861d1d76e559382658ef4d762'
const CHARLIE_PEER_ID_STR: string = '12D3KooWPGsW7vZ8VsmJ9Lws9vsKaBiACZXQ3omRm3rFUho5BpvF'
const CHARLIE_ETHEREUM_ADDR_STR: string = '0xd08933750bffb86861d1d76e559382658ef4d763'

export const ALICE_PEER_ID: PeerId = peerIdFromString(ALICE_PEER_ID_STR)
export const ALICE_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.196/tcp/9091/p2p/${ALICE_PEER_ID_STR}`)
export const ALICE_ETHEREUM_ADDR: Address = Address.from_string(ALICE_ETHEREUM_ADDR_STR)
export const ALICE_OFFCHAIN_PUBKEY: OffchainPublicKey = OffchainPublicKey.from_peerid_str(ALICE_PEER_ID_STR)
export const ALICE_ACCOUNT_ENTRY: AccountEntry = new AccountEntry(
  ALICE_OFFCHAIN_PUBKEY,
  ALICE_ETHEREUM_ADDR.clone(),
  ALICE_MULTI_ADDR.toString(),
  Date.now()
)

export const BOB_PEER_ID: PeerId = peerIdFromString(BOB_PEER_ID_STR)
export const BOB_MULTI_ADDR = new Multiaddr(`/ip4/34.65.237.197/tcp/9091/p2p/${BOB_PEER_ID_STR}`)
export const BOB_ETHEREUM_ADDR: Address = Address.from_string(BOB_ETHEREUM_ADDR_STR)
export const BOB_OFFCHAIN_PUBKEY: OffchainPublicKey = OffchainPublicKey.from_peerid_str(BOB_PEER_ID_STR)
export const BOB_ACCOUNT_ENTRY: AccountEntry = new AccountEntry(
  BOB_OFFCHAIN_PUBKEY,
  BOB_ETHEREUM_ADDR.clone(),
  BOB_MULTI_ADDR.toString(),
  Date.now()
)

export const CHARLIE_PEER_ID: PeerId = peerIdFromString(CHARLIE_PEER_ID_STR)
export const CHARLIE_ETHEREUM_ADDR: Address = Address.from_string(CHARLIE_ETHEREUM_ADDR_STR)
export const CHARLIE_MULTI_ADDR = new Multiaddr(`/ip4/34.65.1.197/tcp/9091/p2p/${CHARLIE_PEER_ID_STR}`)
export const CHARLIE_OFFCHAIN_PUBKEY: OffchainPublicKey = OffchainPublicKey.from_peerid_str(CHARLIE_PEER_ID_STR)
export const CHARLIE_ACCOUNT_ENTRY: AccountEntry = new AccountEntry(
  CHARLIE_OFFCHAIN_PUBKEY,
  CHARLIE_ETHEREUM_ADDR.clone(),
  CHARLIE_MULTI_ADDR.toString(),
  Date.now()
)

export const INVALID_PEER_ID = 'definetly not a valid peerId'

export function channelEntryCreateMock(): ChannelEntry {
  const src = Address.from_string('0x4a34f4c1f1defceaa88f1dd22a8d9c2db70b21eb')
  const dest = Address.from_string('0xe1ad1f04979209f61e64a2a87bde502b465ade50')
  return new ChannelEntry(
    src,
    dest,
    new Balance('1', BalanceType.HOPR),
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
  // set max to 2 msgs
  let cfg = new MessageInboxConfiguration()
  cfg.capacity = 2
  let inbox = new MessageInbox(cfg)
  return {
    api: await setupRestApi(service, '/api/v3', node, inbox, createTestMocks(), {
      disableApiAuthentication: true
    }),
    service,
    inbox
  }
}

/**
 * Creates express app and initializes all routes for testing
 * @returns apiInstance for openAPI validation and express instance for supertest requests with superuser authentication token set to superuser
 */
export const createAuthenticatedTestApiInstance = async (node: any) => {
  const service = express()
  let inbox = new MessageInbox(new MessageInboxConfiguration())
  return {
    api: await setupRestApi(service, '/api/v3', node, inbox, createTestMocks(), {
      apiToken: 'superuser'
    }),
    service,
    inbox
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
