import type { Types } from '@hoprnet/hopr-core-connector-interface';
export declare function Channel(counterparty: Types.AccountId): Uint8Array;
export declare function ChannelKeyParse(arr: Uint8Array): Uint8Array;
export declare function Challenge(channelId: Types.Hash, challenge: Types.Hash): Uint8Array;
export declare function ChallengeKeyParse(arr: Uint8Array): [Types.Hash, Types.Hash];
export declare function ChannelId(signatureHash: Types.Hash): Uint8Array;
export declare function Nonce(channelId: Types.Hash, nonce: Types.Hash): Uint8Array;
export declare function OnChainSecret(): Uint8Array;
export declare function Ticket(channelId: Types.Hash, challenge: Types.Hash): Uint8Array;
