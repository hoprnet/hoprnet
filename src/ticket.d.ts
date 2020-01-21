import { TypeClasses, toU8a } from './types'


export default interface Ticket extends toU8a {
  channelId: TypeClasses.Hash
  challenge: TypeClasses.Hash
  epoch: TypeClasses.TicketEpoch
  amount: TypeClasses.Balance
  winProb: TypeClasses.Hash
  onChainSecret: TypeClasses.Hash

  getEmbeddedFunds(): TypeClasses.Balance
}