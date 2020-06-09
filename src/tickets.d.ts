import type { Hash, Ticket, SignedTicket, Signature } from './types'
import type CoreConnector from '.'

declare namespace Tickets {
  /**
   * Stores signed ticket using channelId & challange.
   * @param coreConnector coreConnector instance
   * @param channelId channel ID hash
   * @param signedTicket the signed ticket to store
   */
  function store(coreConnector: CoreConnector, channelId: Hash, signedTicket: SignedTicket<Ticket, Signature>): Promise<void>

  /**
   * Get stored tickets.
   * @param coreConnector coreConnector instance
   * @param channelId channel ID hash
   * @returns a promise that resolves to a Map of signed tickets keyed by the challange hex value.
   */
  function get(coreConnector: CoreConnector, channelId: Hash): Promise<Map<string, SignedTicket<Ticket, Signature>>>
}

export default Tickets
