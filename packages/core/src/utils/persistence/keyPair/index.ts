export * from './deserialize'
export * from './serialize'

export const KEYPAIR_CIPHER_ALGORITHM = 'chacha20'
export const KEYPAIR_IV_LENGTH = 16
export const KEYPAIR_CIPHER_KEY_LENGTH = 32
export const KEYPAIR_SALT_LENGTH = 32
export const KEYPAIR_SCRYPT_PARAMS = { N: 8192, r: 8, p: 16 }
export const KEYPAIR_PADDING = Buffer.alloc(16, 0x00)
export const KEYPAIR_MESSAGE_DIGEST_ALGORITHM = 'sha256'
