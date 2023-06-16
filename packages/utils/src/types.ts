// NOTE: This is a hacky workaround for the fact that types were originally just in package `utils`
// and now they are between the `utils` and `core`.
// The TS code expects types to be in package `utils`
// This hack can be removed once more code in other packages is fully migrated to Rust

import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

import { core_types_initialize_crate } from '../../core/lib/core_types.js'
core_types_initialize_crate()

export {
  Acknowledgement,
  AccountEntry,
  AcknowledgedTicket,
  AcknowledgementChallenge,
  Address,
  Balance,
  BalanceType,
  ChannelEntry,
  ChannelStatus,
  Challenge,
  HalfKey,
  HalfKeyChallenge,
  Response,
  Snapshot,
  EthereumChallenge,
  Ticket,
  U256,
  Hash,
  PublicKey,
  PendingAcknowledgement,
  Signature,
  ethereum_signed_hash,
  generate_channel_id,
  UnacknowledgedTicket,
  channel_status_to_string,
  random_integer,
  random_big_integer
} from '../../core/lib/core_types.js'
