declare namespace State {
  const SIZE: number
}

declare interface State extends Uint8Array {
  toU8a(): Uint8Array
}

export default State
