import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { Signature, Ticket } from '../types';
import { Uint8ArrayE } from '../types/extended';
declare class SignedTicket extends Uint8ArrayE implements Types.SignedTicket {
    private _ticket?;
    private _signature?;
    private _signer?;
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature?: Signature;
        ticket?: Ticket;
    });
    get ticketOffset(): number;
    get ticket(): Ticket;
    get signatureOffset(): number;
    get signature(): Signature;
    get signer(): Promise<Uint8Array>;
    verify(pubKey: Uint8Array): Promise<boolean>;
    static get SIZE(): number;
    static create(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature?: Signature;
        ticket?: Ticket;
    }): Promise<SignedTicket>;
}
export default SignedTicket;
