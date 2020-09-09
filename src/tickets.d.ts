import type { Public, AcknowledgedTicket } from './types'

declare interface TicketsStatic {
  /**
   * Get all ackmowledged tickets from of the payment channel with
   * counterparty.
   * @param counterparty
   */
  get(counterparty: Public): Promise<AcknowledgedTicket[]>

  /**
   * Retrieves all tickets.
   * @returns a map of acknowledged tickets keyed by the tickets' database key
   */
  getAll(): Promise<Map<string, AcknowledgedTicket>>

  /**
   * Retrieves all tickets created for counterParty.
   * @param counterPartyPubKey counterParty's public key
   * @returns a map of acknowledged tickets keyed by the tickets' challange
   */
  store(counterparty: Public, signedTicket: AcknowledgedTicket): Promise<void>
}

declare var Tickets: TicketsStatic

export { Tickets }
