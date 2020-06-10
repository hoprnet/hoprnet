import type HoprEthereum from '..';
import { SignedTicket, Hash } from '../types';
/**
 * Store and get tickets stored by the node.
 */
declare class Tickets {
    coreConnector: HoprEthereum;
    constructor(coreConnector: HoprEthereum);
    store(channelId: Hash, signedTicket: SignedTicket): Promise<void>;
    get(channelId: Hash): Promise<Map<string, SignedTicket>>;
}
export default Tickets;
