import type { Types } from '@hoprnet/hopr-core-connector-interface';
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
    get channelIdOffset(): number;
    get channelId(): Hash;
    get challengeOffset(): number;
    get challenge(): Hash;
    get epochOffset(): number;
    get epoch(): TicketEpoch;
    get amountOffset(): number;
    get amount(): Balance;
    get winProbOffset(): number;
    get winProb(): Hash;
    get onChainSecretOffset(): number;
    get onChainSecret(): Hash;
    get hash(): Promise<Hash>;
    static get SIZE(): number;
    getEmbeddedFunds(): BN;
    static create(channel: ChannelInstance, amount: Balance, challenge: Hash, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<SignedTicket>;
    static verify(channel: ChannelInstance, signedTicket: SignedTicket): Promise<boolean>;
    static submit(channel: any, signedTicket: SignedTicket): Promise<void>;
}
export default Ticket;
