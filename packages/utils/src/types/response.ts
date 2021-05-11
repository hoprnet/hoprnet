import { Hash } from './primitives'
import { Challenge } from '.'
import type { HalfKey } from '.'

import { publicKeyCreate, privateKeyTweakAdd } from 'secp256k1'

export class Response extends Hash {
  static fromHalfKeys(firstHalfKey: HalfKey, secondHalfKey: HalfKey): Response {
    return new Response(privateKeyTweakAdd(firstHalfKey.clone().serialize(), secondHalfKey.serialize()))
  }

  toChallenge(): Challenge {
    return new Challenge(publicKeyCreate(this.arr))
  }
}
