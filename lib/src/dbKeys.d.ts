import type PeerId from 'peer-id';
export declare function AcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array;
export declare function UnAcknowledgedTickets(publicKeyCounterparty: Uint8Array, id: Uint8Array): Uint8Array;
export declare function UnAcknowledgedTicketsParse(arg: Uint8Array): Promise<[PeerId, Uint8Array]>;
export declare function PacketTag(tag: Uint8Array): Uint8Array;
export declare const KeyPair: Uint8Array;
