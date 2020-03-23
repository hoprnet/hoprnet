import assert from 'assert'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaVerify } from 'secp256k1'

// @ts-ignore
import keccak256 = require('keccak256')

import { PromiEvent, TransactionReceipt } from 'web3-core'
import { BlockTransactionString } from 'web3-eth'
import Web3 from 'web3'
import BN from "bn.js"
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { AccountId, Signature, Hash } from "../types"
import * as constants from '../constants'

export function isPartyA(self: Types.AccountId, counterparty:Types.AccountId): boolean {
  return Buffer.compare(self, counterparty) < 0
}

export function getParties(
  self: Types.AccountId,
  counterparty: Types.AccountId
): [Types.AccountId, Types.AccountId] {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

export function getId(self: Types.AccountId, counterparty: Types.AccountId): Promise<Uint8Array> {
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

export async function pubKeyToAccountId(pubKey: Uint8Array): Promise<Types.AccountId> {
  if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${
        constants.COMPRESSED_PUBLIC_KEY_LENGTH
      }. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`
    )

    return new AccountId((await hash(publicKeyConvert(pubKey, false).slice(1))).slice(12))
}

export function hashSync(msg: Uint8Array): Types.Hash {
  return new Hash(new Uint8Array(keccak256(Buffer.from(msg))))
}

export async function hash(msg: Uint8Array): Promise<Types.Hash> {
  return Promise.resolve(hashSync(msg))
}

export async function sign(msg: Uint8Array, privKey: Uint8Array): Promise<Types.Signature> {
  const result = ecdsaSign(msg, privKey)

  const response = new Signature(undefined, {
    signature: result.signature,
    recovery: result.recovery
  })

  return response
}

export async function verify(msg: Uint8Array, signature: Types.Signature, pubKey: Uint8Array): Promise<boolean> {
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
      .on('error', error => {
        reject(error)
      })
  })
}

export async function wait(ms: number) {
  return new Promise(resolve => {
    setTimeout(resolve, ms)
  })
}

// TODO: only use this during localnet
export async function advanceBlockAtTime(web3: Web3, time: number) {
  return new Promise((resolve, reject) => {
    // @ts-ignore
    web3.currentProvider.send(
      {
        jsonrpc: "2.0",
        method: "evm_mine",
        params: [time],
        id: new Date().getTime(),
      },
      async (err: any) => {
        if (err) {
          return reject(err);
        }
        const newBlock = await web3.eth.getBlock("latest");
        const newBlockHash = newBlock.hash

        return resolve(newBlockHash);
      },
    );
  });
};

export async function waitFor({
  getCurrentBlock,
  web3,
  timestamp
}: {
  getCurrentBlock: () => Promise<BlockTransactionString>,
  web3: Web3,
  timestamp?: number
  // blockNumber?: number
}): Promise<void> {
  const now = await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)

  if (timestamp < now) {
    return undefined;
  }

  await advanceBlockAtTime(web3, Math.ceil(timestamp / 1e3) + 1)

  return waitFor({
    getCurrentBlock,
    web3,
    timestamp
  })
}

// TODO: production code
// export async function waitFor({
//   getCurrentBlock,
//   timestamp
// }: {
//   getCurrentBlock: () => Promise<BlockTransactionString>
//   timestamp?: number
//   // blockNumber?: number
// }): Promise<void> {
//   const now = await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)
//   console.log({
//     now,
//     timestamp,
//     diff: now - timestamp
//   })

//   if (timestamp < now) {
//     return undefined;
//   }

//   const diff = (now - timestamp) || 60 * 1e3

//   await wait(diff)
//   return waitFor({
//     getCurrentBlock,
//     timestamp: await getCurrentBlock().then(block => Number(block.timestamp) * 1e3)
//   })
// }
