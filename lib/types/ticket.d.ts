import type { Types } from "@hoprnet/hopr-core-connector-interface";
import BN from 'bn.js';
import { Hash, TicketEpoch, Balance, SignedTicket } from '.';
import { Uint8ArrayE } from '../types/extended';
import type ChannelInstance from '../channel';
declare class Ticket extends Uint8ArrayE implements Types.Ticket {
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        channelId: Hash;
        challenge: Hash;
        epoch: TicketEpoch;
        amount: Balance;
        winProb: Hash;
        onChainSecret: Hash;
    });
    get channelId(): Hash;
    get challenge(): Hash;
    get epoch(): TicketEpoch;
    get amount(): Balance;
    get winProb(): Hash;
    get onChainSecret(): Hash;
    getEmbeddedFunds(): BN;
    get hash(): Promise<Types.Hash>;
    static get SIZE(): number;
    static create(channel: ChannelInstance, amount: Balance, challenge: Hash): Promise<SignedTicket>;
    static verify(channel: ChannelInstance, signedTicket: SignedTicket): Promise<boolean>;
    static submit(channel: any, signedTicket: SignedTicket): Promise<void>;
}
export default Ticket;
