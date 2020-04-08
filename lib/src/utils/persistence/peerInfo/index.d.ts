/// <reference types="node" />
declare type SerializedPeerId = Buffer;
declare type SerializedMultiaddr = Buffer;
declare type SerializedPublicKey = Buffer;
export declare type SerializedPeerInfo = [SerializedPeerId, SerializedMultiaddr[]] | [SerializedPeerId, SerializedMultiaddr[], SerializedPublicKey];
export * from './serialize';
export * from './deserialize';
