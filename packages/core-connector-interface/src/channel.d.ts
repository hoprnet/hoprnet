import type AcknowledgedTicket from './types/acknowledgedTicket'
import type { Address, Balance, Hash, Public, SignedTicket, ChannelEntry } from './types'
import type Indexer from './indexer'

declare interface ChannelStatic {
  // TODO: remove connector and replace with ethereum global context
  new (connector: any, self: Public, counterparty: Public): Channel
}

declare interface Channel {
  readonly counterparty: Public

  getId(): Promise<Hash>

  getState(): Promise<ChannelEntry>

  getBalances(): Promise<{
    self: Balance
    counterparty: Balance
  }>

  open(fundAmount: Balance): Promise<string>

  initializeClosure(): Promise<string>

  finalizeClosure(): Promise<string>

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
