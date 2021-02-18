import { Hash, SignedTicket } from '.'

declare interface AcknowledgedTicket {
  constructor(
    signedTicket: SignedTicket,
    response: Hash,
    preImage?: Hash
  )

  signedTicket: Promise<SignedTicket>
  signedTicketOffset: number

  response: Hash
  responseOffset: number

  preImage: Hash
  preImageOffset: number

  serialized(): Uint8Array
  SIZE(): number
}

export default AcknowledgedTicket
