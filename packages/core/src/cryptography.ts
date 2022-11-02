// Load `core-crypto` crate
import { set_panic_hook as crypto_core_panic_hook } from '../lib/core_crypto.js'
crypto_core_panic_hook()
export { PRG, PRP } from '../lib/core_crypto.js'
