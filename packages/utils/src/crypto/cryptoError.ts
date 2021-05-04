import { CRYPTO_DEBUG_MODE } from './constants'

export class CryptoError extends Error {
  constructor(msg?: string) {
    super(msg)

    if (!CRYPTO_DEBUG_MODE) {
      Error.captureStackTrace?.(this, () => {})
    }
  }

  get message() {
    if (CRYPTO_DEBUG_MODE) {
      return super.message
    } else {
      return `General error.`
    }
  }

  get stack() {
    if (CRYPTO_DEBUG_MODE) {
      return super.stack
    } else {
      return ''
    }
  }
}
