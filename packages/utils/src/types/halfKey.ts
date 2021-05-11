import { publicKeyCreate } from 'secp256k1'

import { Hash } from './primitives'
import { HalfKeyChallenge } from '.'

export class HalfKey extends Hash {
  toChallenge(): HalfKeyChallenge {
    return new HalfKeyChallenge(publicKeyCreate(this.arr))
  }
}
