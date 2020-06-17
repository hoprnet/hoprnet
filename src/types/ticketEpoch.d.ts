import type BN from 'bn.js'

declare namespace TicketEpoch {
  const SIZE: number
}
declare interface TicketEpoch extends BN {
  toU8a(): Uint8Array
}

export default TicketEpoch
