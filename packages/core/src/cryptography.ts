// Load `core-crypto` crate
import { core_crypto_initialize_crate } from '../lib/core_crypto.js'
core_crypto_initialize_crate()

import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

export { random_float, random_fill, random_bounded_integer } from '../lib/core_crypto.js'
