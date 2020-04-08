import type { Types } from "@hoprnet/hopr-core-connector-interface";
import { Signature, Ticket } from '.';
import { Uint8ArrayE } from '../types/extended';
declare class SignedTicket extends Uint8ArrayE implements Types.SignedTicket<Ticket, Signature> {
    private _ticket?;
    private _signature?;
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature: Signature;
        ticket: Ticket;
    });
    get ticket(): Ticket;
    get signature(): Signature;
    get signer(): Promise<Uint8Array>;
    static get SIZE(): number;
    static create(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature: Signature;
        ticket: Ticket;
    }): SignedTicket;
}
export default SignedTicket;
