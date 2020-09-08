import type { StoredTicket } from './storedTicket'
import { Public } from '../types'

declare interface TicketsStatic {
  /**
   * Stores signed ticket using channelId & challange.
   * @param channelId channel ID hash
   * @param signedTicket the signed ticket to store
   */
  get(counterparty: Public): Promise<Map<string, StoredTicket>>
  /**
   * Get stored tickets.
   * @param channelId channel ID hash
   * @returns a promise that resolves to a Map of signed tickets keyed by the challange hex value.
   */
  store(counterparty: Public, signedTicket: StoredTicket): Promise<void>
}

declare var Tickets: TicketsStatic

export { Tickets }
