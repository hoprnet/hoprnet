import { CurvePoint } from './curvePoint'

import { publicKeyTweakAdd, publicKeyCombine } from 'secp256k1'
import type { HalfKeyChallenge, HalfKey } from '.'
import { EthereumChallenge } from '.'

export class Challenge extends CurvePoint {
  static fromExponent(exponent: Uint8Array): Challenge {
    return new Challenge(CurvePoint.fromExponent(exponent).serialize())
  }

  static fromHintAndShare(ownShare: HalfKeyChallenge, hint: HalfKeyChallenge) {
    return new Challenge(publicKeyCombine([ownShare.serialize(), hint.serialize()]))
  }

  static fromOwnShareAndHalfKey(ownShare: HalfKeyChallenge, halfKey: HalfKey) {
    return new Challenge(publicKeyTweakAdd(ownShare.serialize(), halfKey.serialize()))
  }

  toEthereumChallenge() {
    return new EthereumChallenge(this.toAddress().serialize())
  }
}
