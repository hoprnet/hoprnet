import HoprCoreConnector from '..'
import Hash from './hash'
import SignedTicket from './signedTicket'
import Public from './public'

declare interface AcknowledgedTicketStatic {
  readonly SIZE: number

  create(
    paymentChannels: HoprCoreConnector,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signedTicket: SignedTicket
      response: Hash
      preImage: Hash
      redeemed: boolean
    }
  ): AcknowledgedTicket
}

declare interface AcknowledgedTicket {
  signedTicketOffset: number
  signedTicket: Promise<SignedTicket>
  responseOffset: number
  response: Hash
  preImageOffset: number
  preImage: Hash
  redeemedOffset: number
  redeemed: boolean

  verify(public: Public): Promise<boolean>
  isWinning(): Promise<boolean>
}

declare var AcknowledgedTicket: AcknowledgedTicketStatic

export default AcknowledgedTicket
