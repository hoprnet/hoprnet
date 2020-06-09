import type { Hash, Ticket, SignedTicket, Signature } from './types'
import type HoprCoreConnector from '.'

declare namespace Tickets {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  function store<CoreConnector extends HoprCoreConnector, ConcreteTicket extends Ticket, ConcreteSignature extends Signature>(
    coreConnector: CoreConnector,
    channelId: Hash,
    signedTicket: SignedTicket<ConcreteTicket, ConcreteSignature>
  ): Promise<void>

  function get<CoreConnector extends HoprCoreConnector, ConcreteTicket extends Ticket, ConcreteSignature extends Signature>(
    coreConnector: CoreConnector,
    channelId: Hash
  ): Promise<Map<string, SignedTicket<ConcreteTicket, ConcreteSignature>>>
}

export default Tickets
