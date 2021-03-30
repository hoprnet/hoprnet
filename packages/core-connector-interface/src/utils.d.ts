import type BN from 'bn.js'
import type { Address, Hash, Signature, Balance } from './types'

/**
 * Decides whether we take the role of partyA in the channel with `counterparty`.
 * @param self id of ourself
 * @param counterparty id of the counterparty
 */
export declare function isPartyA(self: Address, counterparty: Address): boolean

/**
 * Returns the Id of the channel between ourself and `counterparty`.
 * @param self id of ourself
 * @param counterparty id of the counterparty
 * @param props additional arguments
 */
export declare function getId(self: Address, counterparty: Address, ...props: any[]): Promise<Hash>

/**
 * Converts a public key into an on-chain Address (e.g. an Ethereum address).
 * @param pubkey a public key
 * @param args additional arguments
 */
export declare function pubKeyToAddress(pubkey: Uint8Array, ...args: any[]): Promise<Address>

/**
 * Uses the native on-chain hash function to compute a hash value of `msg`.
 * @param msg message to hash
 */
export declare function hash(msg: Uint8Array): Promise<Hash>

/**
 * Uses the native on-chain signature scheme to create an on-chain verifiable signature.
 * @param msg message to sign
 * @param privKey private key of the signer
 */
export declare function sign(msg: Uint8Array, privKey: Uint8Array): Promise<Signature>

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

/**
 * Compute the winning probability that is set for a ticket
 * @param prob Desired winning probability of a ticket, e.g. 0.6 resp. 60%
 */
export declare function computeWinningProbability(prob: number): Hash

/**
 * Transforms Uint256 encoded probabilities into floats.
 *
 * @notice mostly used to check a ticket's winning probability.
 *
 * @notice the precision is very limited
 *
 * @param winProb Uint256-encoded version of winning probability
 */
export declare function getWinProbabilityAsFloat(winProb: Hash): number

/**
 * Convert a state counter, to a number represeting the channels iteration.
 * Iteration stands for the amount of times a channel has been opened and closed.
 *
 * @param stateCounter the state count
 * @returns channel's iteration
 */
export declare function stateCounterToIteration(stateCounter: BN): BN
