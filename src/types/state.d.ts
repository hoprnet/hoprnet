import type BN from 'bn.js'

declare interface StateStatic {
  readonly SIZE: number

  new (State: BN, ...props: any[]): State
}
declare interface State extends BN {
  toU8a(): Uint8Array
}

declare var State: StateStatic

export default State
