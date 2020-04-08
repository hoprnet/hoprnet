import type HoprEthereum from "..";
import type { Types } from "@hoprnet/hopr-core-connector-interface";
import { SignedTicket } from '../types';
declare class Ticket {
    static store(coreConnector: HoprEthereum, channelId: Types.Hash, signedTicket: Types.SignedTicket<any, any>): Promise<void>;
    static get(coreConnector: HoprEthereum, channelId: Types.Hash): Promise<Map<string, SignedTicket>>;
}
export default Ticket;
