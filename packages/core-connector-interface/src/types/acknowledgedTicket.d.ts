import { Hash, SignedTicket } from '.'

declare interface AcknowledgedTicketStatic {
  SIZE(): number
}
declare interface AcknowledgedTicket {
  constructor(signedTicket: SignedTicket, response: Hash, preImage?: Hash)

  signedTicket: Promise<SignedTicket>
  signedTicketOffset: number

  response: Hash
  responseOffset: number

  preImage: Hash
  preImageOffset: number

  serialized(): Uint8Array
}

declare var AcknowledgedTicket: AcknowledgedTicketStatic
export default AcknowledgedTicket
