declare namespace State {
  const SIZE: number
}
declare interface State {
  toU8a(): Uint8Array
}

export default State
