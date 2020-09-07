import BN from 'bn.js'

declare interface MomentStatic {
  readonly SIZE: number

  new (moment: BN | number, ...props: any[]): Moment
}
declare interface Moment extends BN {
  toU8a(): Uint8Array
}

declare var Moment: MomentStatic

export default Moment
