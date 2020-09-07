import BN from 'bn.js'

declare interface ChannelStateStatic {
  readonly SIZE: number

  new (channelState: BN | number, ...props: any[]): ChannelState
}
declare interface ChannelState extends BN {
  toU8a(): Uint8Array
}

declare var ChannelState: ChannelStateStatic

export default ChannelState
