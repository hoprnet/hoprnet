import type BN from 'bn.js'

declare namespace TicketEpoch {
  const SIZE: number
}
declare interface TicketEpoch extends BN {
  new (ticketEpoch: BN, ...props: any[]): TicketEpoch

  toU8a(): Uint8Array
}

export default TicketEpoch
