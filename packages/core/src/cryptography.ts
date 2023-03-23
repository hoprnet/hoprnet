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
  derive_commitment_seed,
  IteratedHash,
  Intermediate,
  iterate_hash,
  recover_iterated_hash,
  calculate_mac
} from '../lib/core_crypto.js'
