import { CurvePoint } from './curvePoint.js'

import secp256k1 from 'secp256k1'
import type { HalfKeyChallenge, HalfKey } from './index.js'
import { EthereumChallenge } from './index.js'

export class Challenge extends CurvePoint {
  static fromExponent(exponent: Uint8Array): Challenge {
    return new Challenge(CurvePoint.fromExponent(exponent).serialize())
  }

  static fromHintAndShare(ownShare: HalfKeyChallenge, hint: HalfKeyChallenge) {
    return new Challenge(secp256k1.publicKeyCombine([ownShare.serialize(), hint.serialize()]))
  }

  static fromOwnShareAndHalfKey(ownShare: HalfKeyChallenge, halfKey: HalfKey) {
    return new Challenge(secp256k1.publicKeyTweakAdd(ownShare.serialize(), halfKey.serialize()))
  }

  toEthereumChallenge() {
    return new EthereumChallenge(this.toAddress().serialize())
  }
}
