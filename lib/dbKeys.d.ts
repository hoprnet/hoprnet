import { Hash, AccountId } from './types';
export declare function Channel(counterparty: AccountId): Uint8Array;
export declare function ChannelKeyParse(arr: Uint8Array): Uint8Array;
export declare function Challenge(channelId: Hash, challenge: Hash): Uint8Array;
export declare function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash];
export declare function ChannelId(signatureHash: Hash): Uint8Array;
export declare function Nonce(channelId: Hash, nonce: Hash): Uint8Array;
export declare function OnChainSecret(): Uint8Array;
export declare function Ticket(channelId: Hash, challenge: Hash): Uint8Array;
