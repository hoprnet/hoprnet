import {COMPRESSED_PUBLIC_KEY_LENGTH} from '../constants'
import {Types} from '@hoprnet/hopr-core-connector-interface'
import {privKeyToPubKey, pubKeyToAccountId} from '../utils'
import AccountId from './accountId'
import {Uint8ArrayE} from './extended'

class Public extends Uint8ArrayE implements Types.Public {
  get NAME() {
    return 'Public'
  }

  slice(begin = 0, end = Public.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = Public.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  toAccountId(): Promise<AccountId> {
    return pubKeyToAccountId(this)
  }

  static get SIZE(): number {
    return COMPRESSED_PUBLIC_KEY_LENGTH
  }

  static async fromPrivKey(privKey: Uint8Array): Promise<Public> {
    return new Public(await privKeyToPubKey(privKey))
  }
}

export default Public
