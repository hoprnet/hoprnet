import Hash from './hash'
import TicketEpoch from './ticketEpoch'

declare interface StateStatic {
  readonly SIZE: number

  new (
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      secret: Hash
      epoch: TicketEpoch
    }
  ): State
}
declare interface State extends Uint8Array {
  secret: Hash
  epoch: TicketEpoch
}

declare var State: StateStatic

export default State
