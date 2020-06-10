import { Hash, Balance, SignedTicket } from '../types';
import type Channel from '.';
declare class TicketFactory {
    channel: Channel;
    constructor(channel: Channel);
    create(amount: Balance, challenge: Hash, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<SignedTicket>;
    verify(signedTicket: SignedTicket): Promise<boolean>;
    submit(signedTicket: SignedTicket): Promise<void>;
}
export default TicketFactory;
