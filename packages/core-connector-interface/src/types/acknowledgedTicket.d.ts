import type HoprCoreConnector from '..'
import {Hash, SignedTicket} from '.'

declare interface AcknowledgedTicketStatic {
  SIZE(coreConnector: HoprCoreConnector): number

  create(
    coreConnector: HoprCoreConnector,
    arr?: {bytes: ArrayBuffer; offset: number},
    struct?: {signedTicket?: SignedTicket; response?: Hash; preImage?: Hash; redeemed?: boolean}
  ): AcknowledgedTicket
}

declare interface AcknowledgedTicket {
  signedTicket: Promise<SignedTicket>
  signedTicketOffset: number

  response: Hash
  responseOffset: number

  preImage: Hash
  preImageOffset: number

  redeemed: boolean
  redeemedOffset: number
}

declare var AcknowledgedTicket: AcknowledgedTicketStatic

export default AcknowledgedTicket
