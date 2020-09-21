import * as addresses from '../ethereum/addresses'
import type { TransactionObject } from '../tsc/web3/types'
import assert from 'assert'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaRecover, ecdsaVerify } from 'secp256k1'
import createKeccakHash from 'keccak'
import { PromiEvent, TransactionReceipt, TransactionConfig } from 'web3-core'
import { BlockTransactionString } from 'web3-eth'
import Web3 from 'web3'
import Debug from 'debug'
import { u8aCompare, u8aConcat, u8aEquals, A_STRICLY_LESS_THAN_B, A_EQUALS_B, gcd } from '@hoprnet/hopr-utils'
import { AccountId, Balance, Hash, Signature } from '../types'
import { ContractEventEmitter } from '../tsc/web3/types'
import { ChannelStatus } from '../types/channel'
import * as constants from '../constants'
import * as time from './time'
import * as events from './events'
import BN from 'bn.js'

export { time, events }

/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns true if self is partyA
 */
export function isPartyA(self: AccountId, counterparty: AccountId): boolean {
  return Buffer.compare(self, counterparty) < 0
}

/**
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns an array of partyA's and partyB's accountIds
 */
export function getParties(self: AccountId, counterparty: AccountId): [AccountId, AccountId] {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

/**
 * Get the channel id of self and counterparty
 * @param self our node's accountId
 * @param counterparty counterparty's accountId
 * @returns a promise resolved to Hash
 */
export function getId(self: AccountId, counterparty: AccountId): Promise<Hash> {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH))
}

/**
 * Given a private key, derive public key.
 * @param privKey the private key to derive the public key from
 * @returns a promise resolved to Uint8Array
 */
export async function privKeyToPubKey(privKey: Uint8Array): Promise<Uint8Array> {
  if (privKey.length != constants.PRIVATE_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${
        privKey.length ? ` of length ${privKey.length}` : ''
      }.`
    )

  return publicKeyCreate(privKey)
}

/**
 * Given a public key, derive the AccountId.
 * @param pubKey the public key to derive the AccountId from
 * @returns a promise resolved to AccountId
 */
export async function pubKeyToAccountId(pubKey: Uint8Array): Promise<AccountId> {
  if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${
        constants.COMPRESSED_PUBLIC_KEY_LENGTH
      }. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`
    )

  return new AccountId((await hash(publicKeyConvert(pubKey, false).slice(1))).slice(12))
}

/**
 * Given a message, generate hash using keccak256.
 * @param msg the message to hash
 * @returns a promise resolved to Hash
 */
export async function hash(msg: Uint8Array): Promise<Hash> {
  return Promise.resolve(new Hash(createKeccakHash('keccak256').update(Buffer.from(msg)).digest()))
}

/**
 * Signs a message with ECDSA
 * @param msg the message to sign
 * @param privKey the private key to use when signing
 * @param pubKey deprecated
 * @param arr
 * @returns a promise resolved to Hash
 */
export async function sign(
  msg: Uint8Array,
  privKey: Uint8Array,
  pubKey?: Uint8Array,
  arr?: {
    bytes: ArrayBuffer
    offset: number
  }
): Promise<Signature> {
  if (privKey.length != constants.PRIVATE_KEY_LENGTH) {
    throw Error(
      `Invalid privKey argument. Expected a Uint8Array with ${constants.PRIVATE_KEY_LENGTH} elements but got one with ${privKey.length}.`
    )
  }
  const result = ecdsaSign(msg, privKey)

  const response = new Signature(arr, {
    signature: result.signature,
    recovery: result.recid,
  })

  return response
}

/**
 * Recovers the public key of the signer from the message
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @returns a promise resolved to Uint8Array, the signers public key
 */
export async function signer(msg: Uint8Array, signature: Signature): Promise<Uint8Array> {
  return ecdsaRecover(signature.signature, signature.recovery, msg)
}

/**
 * Checks the validity of the signature by using the public key
 * @param msg the message that was signed
 * @param signature the signature of the signed message
 * @param pubKey the public key of the potential signer
 * @returns a promise resolved to true if the public key provided matches the signer's
 */
export async function verify(msg: Uint8Array, signature: Signature, pubKey: Uint8Array): Promise<boolean> {
  return ecdsaVerify(signature.signature, msg, pubKey)
}

/**
 * Decides whether a ticket is a win or not.
 * Note that this mimics the on-chain logic.
 * @dev Purpose of the function is to check the validity of
 * a ticket before we submit it to the blockchain.
 * @param ticketHash hash value of the ticket to check
 * @param challengeResponse response that solves the signed challenge
 * @param preImage preImage of the current onChainSecret
 * @param winProb winning probability of the ticket
 */
export async function isWinningTicket(ticketHash: Hash, challengeResponse: Hash, preImage: Hash, winProb: Hash) {
  return [A_STRICLY_LESS_THAN_B, A_EQUALS_B].includes(
    u8aCompare(await hash(u8aConcat(ticketHash, preImage, challengeResponse)), winProb)
  )
}

/**
 * Compute the winning probability that is set for a ticket
 * @param prob Desired winning probability of a ticket, e.g. 0.6 resp. 60%
 */
export function computeWinningProbability(prob: number): Uint8Array {
  if (prob == 1) {
    return new Uint8Array(Hash.SIZE).fill(0xff)
  }

  if (prob == 0) {
    return new Uint8Array(Hash.SIZE).fill(0x00)
  }

  let dividend = new BN(prob.toString(2).slice(2), 2)
  let divisor = new BN(0).bincn(prob.toString(2).slice(2).length)

  return new Uint8Array(new BN(0).bincn(256).isubn(1).imul(dividend).div(divisor).toArray('be', Hash.SIZE))
}

/**
 * Checks whether the given response solves a given challenge
 * @param challenge challenge for which we search a preImage
 * @param response response to verify
 */
export async function checkChallenge(challenge: Hash, response: Hash) {
  return u8aEquals(challenge, await hash(response))
}

/**
 * Convert between units'
 * @param amount a BN instance of the amount to be converted
 * @param sourceUnit
 * @param targetUnit
 * @returns a BN instance of the resulted conversion
 */
export function convertUnit(amount: Balance, sourceUnit: 'eth', targetUnit: 'wei'): Balance
export function convertUnit(amount: Balance, sourceUnit: 'wei', targetUnit: 'eth'): Balance
export function convertUnit(amount: Balance, sourceUnit: 'eth' | 'wei', targetUnit: 'eth' | 'wei'): Balance {
  assert(['eth', 'wei'].includes(sourceUnit), 'not implemented')

  if (sourceUnit === 'eth') {
    return Web3.utils.toWei(amount, targetUnit as any) as any
  } else {
    return Web3.utils.fromWei(amount, targetUnit as any) as any
  }
}

/**
 * Wait until block has been confirmed.
 *
 * @typeparam T Our PromiEvent
 * @param event Our event, returned by web3
 * @returns the transaction receipt
 */
export async function waitForConfirmation<T extends PromiEvent<any>>(event: T) {
  return new Promise<TransactionReceipt>((resolve, reject) => {
    return event
      .on('receipt', (receipt) => {
        resolve(receipt)
      })
      .on('error', (err) => {
        const outOfEth = err.message.includes(`enough funds`)
        const outOfHopr = err.message.includes(`SafeERC20:`)

        if (outOfEth) {
          return reject(Error(constants.ERRORS.OOF_ETH))
        } else if (outOfHopr) {
          return reject(Error(constants.ERRORS.OOF_HOPR))
        } else {
          return reject(err)
        }
      })
  })
}

/**
 * An asychronous setTimeout.
 *
 * @param ms milliseconds to wait
 */
export async function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * Wait until timestamp is reached onchain.
 *
 * @param ms milliseconds to wait
 */
export async function waitFor({
  web3,
  network,
  getCurrentBlock,
  timestamp,
}: {
  web3: Web3
  network: addresses.Networks
  getCurrentBlock: () => Promise<BlockTransactionString>
  timestamp?: number
}): Promise<void> {
  const now = await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3)

  if (timestamp < now) {
    return undefined
  }

  const diff = now - timestamp || 60

  if (network === 'private') {
    await time.increase(web3, diff)
  } else {
    await wait(diff * 1e3)
  }

  return waitFor({
    web3,
    network,
    getCurrentBlock,
    timestamp: await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3),
  })
}

/**
 * Get chain ID.
 *
 * @param web3 a web3 instance
 * @returns the chain ID
 */
export async function getChainId(web3: Web3): Promise<number> {
  return web3.eth.getChainId()
}

/**
 * Get current network's name.
 *
 * @param web3 a web3 instance
 * @returns the network's name
 */
export function getNetworkName(chainId: number): addresses.Networks {
  switch (chainId) {
    case 1:
      return 'mainnet'
    case 2:
      return 'morden'
    case 3:
      return 'ropsten'
    case 4:
      return 'rinkeby'
    case 5:
      return 'goerli'
    case 42:
      return 'kovan'
    case 77:
      return 'solkol'
    case 100:
      return 'xdai'
    default:
      return 'private'
  }
}

/**
 * Convert a state count (one received from on-chain),
 * to an enumarated representation.
 *
 * @param stateCount the state count
 * @returns ChannelStatus
 */
export function stateCountToStatus(stateCount: number): ChannelStatus {
  const status = Number(stateCount) % 10

  if (status >= Object.keys(ChannelStatus).length) {
    throw Error("status like this doesn't exist")
  }

  return status
}

/**
 * A signer factory that signs transactions using the given private key.
 *
 * @param web3 a web3 instance
 * @param privKey the private key to sign transactions with
 * @returns signer
 */
// @TODO: switch to web3js-accounts wallet if it's safe
export function TransactionSigner(web3: Web3, privKey: Uint8Array) {
  const privKeyStr = new Hash(privKey).toHex()

  return async function signTransaction<T>(
    // return of our contract method in web3.Contract instance
    txObject: TransactionObject<T>,
    // config put in .send
    txConfig: TransactionConfig
  ) {
    const abi = txObject.encodeABI()
    // estimation is not always right, adding some more
    // const estimatedGas = Math.floor((await txObject.estimateGas()) * 1.25)
    const estimatedGas = 200e3
    const estimatedGasPrice = 1e9

    // @TODO: provide some of the values to avoid multiple calls
    const signedTransaction = await web3.eth.accounts.signTransaction(
      {
        gas: estimatedGas,
        gasPrice: estimatedGasPrice,
        ...txConfig,
        data: abi,
      },
      privKeyStr
    )

    function send() {
      if (signedTransaction.rawTransaction == null) {
        throw Error(`Cannot process transaction because Web3.js did not give us the raw transaction.`)
      }

      return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction)
    }

    return {
      send,
      transactionHash: signedTransaction.transactionHash,
    }
  }
}

/**
 * Create a prefixed Debug instance.
 *
 * @param prefixes an array containing prefixes
 * @returns a debug instance prefixed by joining 'prefixes'
 */
export function Log(prefixes: string[] = []) {
  return Debug(['hopr-core-ethereum'].concat(prefixes).join(':'))
}

/**
 * Once function 'fn' resolves, remove all listeners from 'event'.
 *
 * @typeparam E Our contract event emitteer
 * @typeparam R fn's return
 * @param event an event
 * @param fn a function to wait for
 */
export async function cleanupPromiEvent<E extends ContractEventEmitter<any>, R extends Promise<any>>(
  event: E,
  fn: (event: E) => R
): Promise<R> {
  return fn(event).finally(() => event.removeAllListeners())
}

/**
 * Get r,s,v values of a signature
 */
export function getSignatureParameters(
  signature: Signature
): {
  r: Uint8Array
  s: Uint8Array
  v: number
} {
  return {
    r: signature.signature.slice(0, 32),
    s: signature.signature.slice(32, 64),
    v: signature.recovery,
  }
}

/**
 * Create a challenge by concatinating and then hashing the secrets.
 * @param secretA
 * @param secretB
 * @returns a promise that resolves to a hash
 */
export async function createChallenge(secretA: Uint8Array, secretB: Uint8Array): Promise<Hash> {
  return await hash(await hash(u8aConcat(secretA, secretB)))
}
