declare interface StateStatic {
  readonly SIZE: number
}

declare interface State extends Uint8Array {
  toU8a(): Uint8Array
}

declare var State: StateStatic

export default State
