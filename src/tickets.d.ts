import type { Public, AcknowledgedTicket } from './types'

declare interface Tickets {
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
  get(counterPartyPubKey: Public): Promise<Map<string, AcknowledgedTicket>>
}

export default Tickets
