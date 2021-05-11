import { CurvePoint } from './curvePoint'

export class Challenge extends CurvePoint {
  toEthereumChallenge() {
    return this.toAddress()
  }
}
