import BN from 'bn.js'

declare interface ChannelStateStatic {
  readonly SIZE: number
  new (state: number): ChannelState
}
declare interface ChannelState extends Uint8Array {
  toBN(): BN
}

declare var ChannelState: ChannelStateStatic

export default ChannelState
