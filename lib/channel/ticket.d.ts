import type IChannel from '.';
import { Hash, Balance, SignedTicket } from '../types';
declare class TicketFactory {
    channel: IChannel;
    constructor(channel: IChannel);
    create(amount: Balance, challenge: Hash, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<SignedTicket>;
    verify(signedTicket: SignedTicket): Promise<boolean>;
    submit(signedTicket: SignedTicket, secretA: Uint8Array, secretB: Uint8Array): Promise<void>;
}
export default TicketFactory;
