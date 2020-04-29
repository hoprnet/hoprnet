import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { TicketEpoch, Hash, Public } from '.';
import { Uint8ArrayE } from '../types/extended';
declare class State extends Uint8ArrayE implements Types.State {
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        secret: Hash;
        pubkey: Public;
        epoch: TicketEpoch;
    });
    get secret(): Hash;
    get pubkey(): Public;
    get epoch(): TicketEpoch;
    static get SIZE(): number;
}
export default State;
