// Load `core-crypto` crate
import { core_crypto_initialize_crate } from '../lib/core_crypto.js'
core_crypto_initialize_crate()

import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

export {
  derive_packet_tag,
  derive_mac_key,
  derive_commitment_seed,
  IteratedHash,
  Intermediate,
  create_tagged_mac,
  Challenge,
  CurvePoint,
  HalfKey,
  HalfKeyChallenge,
  Hash,
  PublicKey,
  Signature,
  random_float,
  random_fill,
  random_bounded_integer,
  GroupElement
} from '../lib/core_crypto.js'
