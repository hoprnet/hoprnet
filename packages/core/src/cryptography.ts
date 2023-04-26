// Load `core-crypto` crate
import { core_crypto_set_panic_hook } from '../lib/core_crypto.js'

import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

core_crypto_set_panic_hook()
export {
  PRG,
  PRGParameters,
  PRP,
  PRPParameters,
  SharedKeys,
  derive_packet_tag,
  derive_mac_key,
  derive_commitment_seed,
  IteratedHash,
  Intermediate,
  iterate_hash,
  recover_iterated_hash,
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
