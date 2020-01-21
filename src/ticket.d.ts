import { TypeClasses } from './types'

export default interface Ticket extends Uint8Array {
  channelId: TypeClasses.Hash
  challenge: TypeClasses.Hash
  epoch: TypeClasses.TicketEpoch
  amount: TypeClasses.Balance
  winProb: TypeClasses.Hash
  onChainSecret: TypeClasses.Hash

  getEmbeddedFunds(): TypeClasses.Balance
}

