import { Hash, SignedTicket } from '../types'

declare interface AcknowledgedTicket {
  signedTicket: Promise<SignedTicket>
  response: Hash
  preImage: Hash
  redeemed: boolean
}

export default AcknowledgedTicket
