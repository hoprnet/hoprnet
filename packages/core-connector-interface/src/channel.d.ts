import type AcknowledgedTicket from './types/acknowledgedTicket'
import type { Address, Balance, Hash, Public, SignedTicket, ChannelEntry } from './types'
import type Indexer from './indexer'

declare interface ChannelStatic {
  new (indexer: Indexer, connector: any, self: Public, counterparty: Public): Channel
}

declare interface Channel {
  readonly counterparty: Public

  getId(): Promise<Hash>

  getState(): Promise<ChannelEntry>

  getBalances(): Promise<{
    self: Balance
    counterparty: Balance
  }>

  open(fundAmount: Balance): Promise<void>

  initializeClosure(): Promise<void>
  finalizeClosure(): Promise<void>

  createTicket(amount: Balance, challenge: Hash, winProb: number): Promise<SignedTicket>

  createDummyTicket(challenge: Hash): Promise<SignedTicket>

  submitTicket(
    ticket: AcknowledgedTicket,
    ticketIndex: Uint8Array
  ): Promise<
    | {
        status: 'SUCCESS'
        receipt: string
        ackTicket: AcknowledgedTicket
      }
    | {
        status: 'FAILURE'
        message: string
      }
    | {
        status: 'ERROR'
        error: Error | string
      }
  >
}

declare var Channel: ChannelStatic

export default Channel
