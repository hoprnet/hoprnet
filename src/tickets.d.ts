import type { SignedTicket, Hash } from './types'

declare interface Tickets {
  /**
   * Stores signed ticket using channelId & challange.
   * @param channelId channel ID hash
   * @param signedTicket the signed ticket to store
   */
  get(channelId: Hash): Promise<Map<string, SignedTicket>>
  /**
   * Get stored tickets.
   * @param channelId channel ID hash
   * @returns a promise that resolves to a Map of signed tickets keyed by the challange hex value.
   */
  store(channelId: Hash, signedTicket: SignedTicket): Promise<void>
}

export default Tickets
