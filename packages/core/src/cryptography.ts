// Load `core-crypto` crate
import { set_panic_hook as crypto_core_panic_hook } from '../lib/core_crypto.js'

import { webcrypto } from 'node:crypto'
// @ts-ignore
globalThis.crypto = webcrypto

crypto_core_panic_hook()
export { PRG, PRP, SharedKeys } from '../lib/core_crypto.js'
