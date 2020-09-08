import { SignedTicket, Hash } from '../types'

declare interface StoredTicket {
  signedTicket: SignedTicket
  response: Hash
  preImage: Hash
  redeemed: boolean
}

export { StoredTicket }
