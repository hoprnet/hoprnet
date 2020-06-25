import type { Types } from '@hoprnet/hopr-core-connector-interface';
import BN from 'bn.js';
import { Hash, TicketEpoch, Balance, Signature } from '.';
import { Uint8ArrayE } from '../types/extended';
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
    get hash(): Hash;
    static get SIZE(): number;
    getEmbeddedFunds(): BN;
    sign(privKey: Uint8Array, pubKey: Uint8Array, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<Signature>;
    static create(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        channelId: Hash;
        challenge: Hash;
        epoch: TicketEpoch;
        amount: Balance;
        winProb: Hash;
        onChainSecret: Hash;
    }): Ticket;
}
export default Ticket;
