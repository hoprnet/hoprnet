import { BYTES32 } from './solidity'
import { COMPRESSED_PUBLIC_KEY_LENGTH } from '../constants'
import { Types } from '@hoprnet/hopr-core-connector-interface'
import { privKeyToPubKey, pubKeyToAccountId } from '../utils'
import AccountId from './accountId'

class Public extends BYTES32 implements Types.Public {
  get NAME() {
    return 'Public'
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
