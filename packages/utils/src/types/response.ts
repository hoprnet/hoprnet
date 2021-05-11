import { Hash } from './primitives'
import { Challenge } from '.'

import { publicKeyCreate } from 'secp256k1'

export class Response extends Hash {
  toChallenge(): Challenge {
    return new Challenge(publicKeyCreate(this.arr))
  }
}
