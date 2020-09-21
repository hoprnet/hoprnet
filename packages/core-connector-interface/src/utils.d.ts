import type { AccountId, Hash, Signature, Balance } from './types'

/**
 * Decides whether we take the role of partyA in the channel with `counterparty`.
 * @param self id of ourself
 * @param counterparty id of the counterparty
 */
export declare function isPartyA(self: AccountId, counterparty: AccountId): boolean

/**
 * Returns the Id of the channel between ourself and `counterparty`.
 * @param self id of ourself
 * @param counterparty id of the counterparty
 * @param props additional arguments
 */
export declare function getId(self: AccountId, counterparty: AccountId, ...props: any[]): Promise<Hash>

/**
 * Converts a public key into an on-chain AccountId (e.g. an Ethereum address).
 * @param pubkey a public key
 * @param args additional arguments
 */
export declare function pubKeyToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<AccountId>

/**
 * Uses the native on-chain hash function to compute a hash value of `msg`.
 * @param msg message to hash
 */
export declare function hash(msg: Uint8Array): Promise<Hash>

/**
 * Uses the native on-chain signature scheme to create an on-chain verifiable signature.
 * @param msg message to sign
 * @param privKey private key of the signer
 * @param pubKey public key of the signer
 * @param arr optional memory for the signature
 */
export declare function sign(
  msg: Uint8Array,
  privKey: Uint8Array,
  pubKey: Uint8Array | undefined,
  arr?: {
    bytes: ArrayBuffer
    offset: number
  }
): Promise<Signature>

/**
 * Uses the native on-chain signature scheme to check a signature for its validity.
 * @param msg message to verify
 * @param signature signature over `msg` to verify
 * @param pubkey public key of the signer
 */
export declare function verify(msg: Uint8Array, signature: Signature, pubkey: Uint8Array): Promise<boolean>

/**
 * Takes an amount and converts it from one unit to another one.
 * @param amount to convert
 * @param sourceUnit unit of `amount`
 * @param targetUnit desired unit of the result
 * @example
 * ```
 * fromUnit('1000000000000000000', 'wei', 'ether') == '1'
 * fromUnit('1', 'ether', 'wei') == '1000000000000000000'
 * ```
 */
export declare function convertUnit(amount: Balance, sourceUnit: string, targetUnit: string): Balance
