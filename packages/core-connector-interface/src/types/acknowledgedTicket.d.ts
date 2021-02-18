import { Hash, SignedTicket } from '.'

declare interface AcknowledgedTicketStatic {
  SIZE(): number
  deserialize(Uint8Array): Promise<AcknowledgedTicket>
}

declare interface AcknowledgedTicket {
  constructor(signedTicket: SignedTicket, response: Hash, preImage?: Hash)
  getSignedTicket(): SignedTicket
  getResponse(): Hash
  getPreImage(): Hash
  setPreImage(Hash)
  serialize(): Uint8Array
}

declare var AcknowledgedTicket: AcknowledgedTicketStatic
export default AcknowledgedTicket
