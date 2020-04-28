import type { Networks } from '../tsc/types'
import type { TransactionObject } from '../tsc/web3/types'

import assert from 'assert'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaRecover, ecdsaVerify } from 'secp256k1'
import createKeccakHash from 'keccak'
import { PromiEvent, TransactionReceipt, TransactionConfig } from 'web3-core'
import { BlockTransactionString } from 'web3-eth'
import Web3 from 'web3'
import BN from 'bn.js'
import Debug from 'debug'
import { AccountId, Signature, Hash } from '../types'
import { ChannelStatus } from '../types/channel'
import * as constants from '../constants'

export function isPartyA(self: AccountId, counterparty: AccountId): boolean {
  return Buffer.compare(self, counterparty) < 0
}

export function getParties(self: AccountId, counterparty: AccountId): [AccountId, AccountId] {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

export function getId(self: AccountId, counterparty: AccountId): Promise<Hash> {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH))
}

export async function privKeyToPubKey(privKey: Uint8Array): Promise<Uint8Array> {
  if (privKey.length != constants.PRIVATE_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${
        privKey.length ? ` of length ${privKey.length}` : ''
      }.`
    )

  return publicKeyCreate(privKey)
}

export async function pubKeyToAccountId(pubKey: Uint8Array): Promise<AccountId> {
  if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${
        constants.COMPRESSED_PUBLIC_KEY_LENGTH
      }. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`
    )

  return new AccountId((await hash(publicKeyConvert(pubKey, false).slice(1))).slice(12))
}

export async function hash(msg: Uint8Array): Promise<Hash> {
  return Promise.resolve(new Hash(createKeccakHash('keccak256').update(Buffer.from(msg)).digest()))
}

export async function sign(msg: Uint8Array, privKey: Uint8Array, pubKey?: Uint8Array, arr?: {
  bytes: ArrayBuffer,
  offset: number
}): Promise<Signature> {
  const result = ecdsaSign(msg, privKey)

  const response = new Signature(arr, {
    signature: result.signature,
    // @ts-ignore-next-line
    recovery: result.recid
  })

  return response
}

export async function signer(msg: Uint8Array, signature: Signature): Promise<Uint8Array> {
  return ecdsaRecover(signature.signature, signature.recovery, msg)
}

export async function verify(msg: Uint8Array, signature: Signature, pubKey: Uint8Array): Promise<boolean> {
  return ecdsaVerify(signature.signature, msg, pubKey)
}

export function convertUnit(amount: BN, sourceUnit: string, targetUnit: 'eth' | 'wei'): BN {
  assert(['eth', 'wei'].includes(sourceUnit), 'not implemented')

  if (sourceUnit === 'eth') {
    return Web3.utils.toWei(amount, targetUnit as any) as any
  } else {
    return Web3.utils.fromWei(amount, targetUnit as any) as any
  }
}

export async function waitForConfirmation<T extends PromiEvent<any>>(event: T) {
  return new Promise<TransactionReceipt>((resolve, reject) => {
    return event
      .on('receipt', receipt => {
        resolve(receipt)
      })
      .on("error", err => {
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

export function advanceBlockAtTime(web3: Web3, time: number): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    // @ts-ignore
    web3.currentProvider.send(
      {
        jsonrpc: '2.0',
        method: 'evm_mine',
        params: [time],
        id: new Date().getTime()
      },
      async (err: any) => {
        if (err) {
          return reject(err)
        }
        const newBlock = await web3.eth.getBlock('latest')
        const newBlockHash = newBlock.hash

        return resolve(newBlockHash)
      }
    )
  })
}

export async function wait(ms: number) {
  return new Promise(resolve => {
    setTimeout(resolve, ms)
  })
}

export async function waitFor({
  web3,
  network,
  getCurrentBlock,
  timestamp
}: {
  web3: Web3
  network: Networks
  getCurrentBlock: () => Promise<BlockTransactionString>
  timestamp?: number
}): Promise<void> {
  const now = await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)

  if (timestamp < now) {
    return undefined
  }

  if (network === 'private') {
    await advanceBlockAtTime(web3, Math.ceil(timestamp / 1e3) + 1)
  } else {
    const diff = now - timestamp || 60 * 1e3
    await wait(diff)
  }

  return waitFor({
    web3,
    network,
    getCurrentBlock,
    timestamp: await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)
  })
}

/*
  return network name, not using web3 'getNetworkType' because
  it misses networks & uses genesis block to determine networkid.
  supports all infura networks
*/
export async function getNetworkId(web3: Web3): Promise<Networks> {
  return web3.eth.net.getId().then(netId => {
    switch (netId) {
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
      default:
        return 'private'
    }
  })
}

export function stateCountToStatus(stateCount: number): ChannelStatus {
  const status = Number(stateCount) % 10

  if (status >= Object.keys(ChannelStatus).length) {
    throw Error("status like this doesn't exist")
  }

  return status
}

// sign transaction's locally and send them
// @TODO: switch to web3js-accounts wallet if it's safe
// @TODO: remove explicit any
export function TransactionSigner(web3: Web3, privKey: Uint8Array): any {
  const privKeyStr = new Hash(privKey).toHex()

  return async function signTransaction<T extends any>(
    // return of our contract method in web3.Contract instance
    txObject: TransactionObject<T>,
    // config put in .send
    txConfig: TransactionConfig
  ) {
    const abi = txObject.encodeABI()
    // estimation is not always right, adding some more
    // const estimatedGas = Math.floor((await txObject.estimateGas()) * 1.25)
    const estimatedGas = 200e3

    // @TODO: provide some of the values to avoid multiple calls
    const signedTransaction = await web3.eth.accounts.signTransaction(
      {
        gas: estimatedGas,
        ...txConfig,
        data: abi
      },
      privKeyStr
    )

    function send() {
      return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction)
    }

    return {
      send,
      transactionHash: signedTransaction.transactionHash
    }
  }
}

export function Log(suffixes: string[] = []) {
  return Debug(["hopr-core-ethereum"].concat(suffixes).join(":"))
}