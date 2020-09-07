import type BN from 'bn.js'

declare interface TicketEpochStatic {
  readonly SIZE: number

  new (ticketEpoch: BN | number, ...props: any[]): TicketEpoch
}
declare interface TicketEpoch extends BN {
  toU8a(): Uint8Array
}

declare var TicketEpoch: TicketEpochStatic

export default TicketEpoch
