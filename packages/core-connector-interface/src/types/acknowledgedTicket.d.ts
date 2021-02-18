import { Hash, SignedTicket } from '.'

declare interface AcknowledgedTicketStatic {
  SIZE(): number
  deserialize(Uint8Array): Promise<AcknowledgedTicket>
}

/*
 * An acknowledged ticket encapsulates the knowledge we have about a ticket
 * that has been successfully acknowledged by a counterparty, and is waiting
 * for us to redeem it.
 *
 * We don't need to store unsuccessful tickets, or tickets that have been
 * redeemed.
 */
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
