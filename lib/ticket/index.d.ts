import type HoprEthereum from "..";
import { SignedTicket, Hash } from '../types';
declare class Ticket {
    static store(coreConnector: HoprEthereum, channelId: Hash, signedTicket: SignedTicket): Promise<void>;
    static get(coreConnector: HoprEthereum, channelId: Hash): Promise<Map<string, SignedTicket>>;
}
export default Ticket;
