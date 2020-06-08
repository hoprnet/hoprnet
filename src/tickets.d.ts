import type { SignedTicket, Hash } from './types'

declare interface Tickets {
  get(channelId: Hash): Promise<Map<string, SignedTicket>>
  store(channelId: Hash, signedTicket: SignedTicket): Promise<void>
}
