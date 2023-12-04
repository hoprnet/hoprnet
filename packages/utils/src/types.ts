// NOTE: This is a hacky workaround for the fact that types were originally just in package `utils`
// and now they are between the `utils` and `core`.
// The TS code expects types to be in package `utils`
// This hack can be removed once more code in other packages is fully migrated to Rust

import { registerMetricsCollector } from './index.js'

import { hoprd_hoprd_initialize_crate, hoprd_hoprd_gather_metrics } from '../../hoprd/lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()
registerMetricsCollector(hoprd_hoprd_gather_metrics)

import { webcrypto } from 'node:crypto'

// @ts-ignore
globalThis.crypto = webcrypto

// Generated with: sed -En 's/^export ((class)|(enum)|(function)) ([A-Za-z0-9_]+).+$/\5,/p' hoprd_hoprd.d.ts | sort | grep -vE "(initialize)|(panic)"
export {
  AccountEntry,
  AcknowledgedTicket,
  Address,
  AnnouncementData,
  ApplicationData,
  AuthorizationToken,
  Balance,
  BalanceType,
  ChainKeypair,
  Challenge,
  ChannelEntry,
  ChannelStatus,
  channel_status_to_number,
  channel_status_to_string,
  ChannelDirection,
  OpenChannelResult,
  CloseChannelResult,
  core_network_gather_metrics,
  core_packet_gather_metrics,
  core_protocol_gather_metrics,
  create_counter,
  create_gauge,
  create_histogram,
  create_histogram_with_buckets,
  create_multi_counter,
  create_multi_gauge,
  create_multi_histogram,
  create_multi_histogram_with_buckets,
  CurvePoint,
  Database,
  derive_commitment_seed,
  derive_mac_key,
  derive_packet_tag,
  EthereumChallenge,
  gather_all_metrics,
  generate_channel_id,
  get_package_version,
  HalfKey,
  HalfKeyChallenge,
  Hash,
  Health,
  health_to_string,
  HeartbeatConfig,
  hoprd_hoprd_gather_metrics,
  HoprKeys,
  IdentityOptions,
  KeyBinding,
  KeyPair,
  merge_encoded_metrics,
  MessageInbox,
  MessageInboxConfiguration,
  MixerConfig,
  MultiCounter,
  MultiGauge,
  MultiHistogram,
  number_to_channel_status,
  OffchainKeypair,
  OffchainPublicKey,
  OffchainSignature,
  PacketInteractionConfig,
  TransportPath,
  PeerOrigin,
  PeerStatus,
  PublicKey,
  random_big_integer,
  random_bounded_integer,
  random_fill,
  random_float,
  random_integer,
  Response,
  Signature,
  SimpleCounter,
  SimpleGauge,
  SimpleHistogram,
  SimpleTimer,
  SmartContractInfo,
  Snapshot,
  Ticket,
  U256,
  WasmVecAccountEntry,
  WasmVecAcknowledgedTicket,
  WasmVecAddress,
  WasmVecChannelEntry,
  HoprTransport,
  PingConfig
} from '../../hoprd/lib/hoprd_hoprd.js'
