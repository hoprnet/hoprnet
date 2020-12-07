import { publicKeyConvert } from 'secp256k1'
import { stringToU8a } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

export const vmErrorMessage = (error: string) => {
  return `VM Exception while processing transaction: revert ${error}`
}

export const splitPubKey = (pubKey: string) => {
  const ACCOUNT_A_PUBKEY = publicKeyConvert(stringToU8a(pubKey), false).slice(1)
  const firstHalf = new BN(ACCOUNT_A_PUBKEY.slice(0, 32))
  const secondHalf = new BN(ACCOUNT_A_PUBKEY.slice(32, 64))

  return {
    firstHalf,
    secondHalf
  }
}
