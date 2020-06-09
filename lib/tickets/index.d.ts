import type HoprEthereum from '..';
import { SignedTicket, Hash } from '../types';
/**
 * Store and get tickets stored by the node.
 */
declare class Tickets {
    static store(coreConnector: HoprEthereum, channelId: Hash, signedTicket: SignedTicket): Promise<void>;
    static get(coreConnector: HoprEthereum, channelId: Hash): Promise<Map<string, SignedTicket>>;
}
export default Tickets;
