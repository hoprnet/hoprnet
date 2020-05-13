import type { addresses } from '@hoprnet/hopr-ethereum';
import type { TransactionObject } from '../tsc/web3/types';
import { PromiEvent, TransactionReceipt, TransactionConfig } from 'web3-core';
import { BlockTransactionString } from 'web3-eth';
import Web3 from 'web3';
import BN from 'bn.js';
import Debug from 'debug';
import { AccountId, Signature, Hash } from '../types';
import { ContractEventEmitter } from '../tsc/web3/types';
import { ChannelStatus } from '../types/channel';
import * as time from './time';
export { time };
/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns true if self is partyA
 */
export declare function isPartyA(self: AccountId, counterparty: AccountId): boolean;
/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns an array of partyA's and partyB's accountIds
 */
export declare function getParties(self: AccountId, counterparty: AccountId): [AccountId, AccountId];
/**
 * Get the channel id of self and counterparty
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns a promise resolved to Hash
 */
export declare function getId(self: AccountId, counterparty: AccountId): Promise<Hash>;
/**
 * Given a private key, derive public key.
 * @param privKey the private key to derive the public key from
 * @returns a promise resolved to Uint8Array
 */
export declare function privKeyToPubKey(privKey: Uint8Array): Promise<Uint8Array>;
/**
 * Given a public key, derive the AccountId.
 * @param pubKey the public key to derive the AccountId from
 * @returns a promise resolved to AccountId
 */
export declare function pubKeyToAccountId(pubKey: Uint8Array): Promise<AccountId>;
/**
 * Given a message, generate hash using keccak256.
 * @param msg the message to hash
 * @returns a promise resolved to Hash
 */
export declare function hash(msg: Uint8Array): Promise<Hash>;
/**
 * Sign message.
 * @param msg the message to sign
 * @param privKey the private key to use when signing
 * @param pubKey deprecated
 * @param arr
 * @returns a promise resolved to Hash
 */
export declare function sign(msg: Uint8Array, privKey: Uint8Array, pubKey?: Uint8Array, arr?: {
    bytes: ArrayBuffer;
    offset: number;
}): Promise<Signature>;
/**
 * Recover signer.
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @returns a promise resolved to Uint8Array, the signers public key
 */
export declare function signer(msg: Uint8Array, signature: Signature): Promise<Uint8Array>;
/**
 * Verify signer.
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @param pubKey the public key of the potential signer
 * @returns a promise resolved to true if the public key provided matches the signer's
 */
export declare function verify(msg: Uint8Array, signature: Signature, pubKey: Uint8Array): Promise<boolean>;
/**
 * Convert between units'
 * @param amount a BN instance of the amount to be converted
 * @param sourceUnit
 * @param targetUnit
 * @returns a BN instance of the resulted conversion
 */
export declare function convertUnit(amount: BN, sourceUnit: 'eth', targetUnit: 'wei'): BN;
export declare function convertUnit(amount: BN, sourceUnit: 'wei', targetUnit: 'eth'): BN;
/**
 * Wait until block has been confirmed.
 *
 * @typeparam T Our PromiEvent
 * @param event Our event, returned by web3
 * @returns the transaction receipt
 */
export declare function waitForConfirmation<T extends PromiEvent<any>>(event: T): Promise<TransactionReceipt>;
/**
 * An asychronous setTimeout.
 *
 * @param ms milliseconds to wait
 */
export declare function wait(ms: number): Promise<unknown>;
/**
 * Wait until timestamp is reached onchain.
 *
 * @param ms milliseconds to wait
 */
export declare function waitFor({ web3, network, getCurrentBlock, timestamp, }: {
    web3: Web3;
    network: addresses.Networks;
    getCurrentBlock: () => Promise<BlockTransactionString>;
    timestamp?: number;
}): Promise<void>;
/**
 * Get current network's name.
 *
 * @param web3 a web3 instance
 * @returns the network's name
 */
export declare function getNetworkId(web3: Web3): Promise<addresses.Networks>;
/**
 * Convert a state count (one received from on-chain),
 * to an enumarated representation.
 *
 * @param stateCount the state count
 * @returns ChannelStatus
 */
export declare function stateCountToStatus(stateCount: number): ChannelStatus;
/**
 * A signer factory that signs transactions using the given private key.
 *
 * @param web3 a web3 instance
 * @param privKey the private key to sign transactions with
 * @returns signer
 */
export declare function TransactionSigner(web3: Web3, privKey: Uint8Array): <T extends unknown>(txObject: TransactionObject<T>, txConfig: TransactionConfig) => Promise<{
    send: () => PromiEvent<import("web3-core-helpers").TransactionRevertInstructionError | TransactionReceipt>;
    transactionHash: string;
}>;
/**
 * Create a prefixed Debug instance.
 *
 * @param prefixes an array containing prefixes
 * @returns a debug instance prefixed by joining 'prefixes'
 */
export declare function Log(prefixes?: string[]): Debug.Debugger;
/**
 * Once function 'fn' resolves, remove all listeners from 'event'.
 *
 * @typeparam E Our contract event emitteer
 * @typeparam R fn's return
 * @param event an event
 * @param fn a function to wait for
 */
export declare function cleanupPromiEvent<E extends ContractEventEmitter<any>, R extends Promise<any>>(event: E, fn: (event: E) => R): Promise<R>;
