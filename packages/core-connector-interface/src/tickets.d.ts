import type { AcknowledgedTicket, Public } from './types'

// @TODO: still under development, might be moved to `hopr-core`
declare interface TicketsStatic {
  /**
   * Get all ackmowledged tickets of the payment channel with
   * counterparty.
   * @param counterparty
   */
  get(counterparty: Public): Promise<Map<string, AcknowledgedTicket>>

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
  store(counterparty: Public, ticket: AcknowledgedTicket): Promise<void>
}

declare var Tickets: TicketsStatic

export { Tickets }
