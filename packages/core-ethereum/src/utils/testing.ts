import type { HoprToken } from '../contracts'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import CoreConnector from '..'
import { BigNumberish, providers as IProviders, ethers } from 'ethers'

export async function fundAccount(funder: ethers.Wallet, token: HoprToken, receiver: string) {
  const amount = ethers.utils.parseEther('1')

  await funder.sendTransaction({
    to: receiver,
    value: amount
  })

  await token.connect(funder).mint(receiver, amount, ethers.constants.HashZero, ethers.constants.HashZero)
}

/**
 * Given a private key, create a connector node.
 * @deprecated
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export async function createNode(privKey: Uint8Array, maxConfirmations: number = 0): Promise<CoreConnector> {
  return CoreConnector.create(new LevelUp(Memdown()), privKey, {
    maxConfirmations
  })
}

// /**
//  * Disconnect web3 as if it lost connection
//  * @param web3 Web3 instance
//  */
// export async function disconnectWeb3(web3: Web3): Promise<void> {
//   try {
//     // @ts-ignore
//     return web3.currentProvider.disconnect(4000)
//   } catch (err) {
//     // console.error(err)
//   }
// }

// advances by one block
export const advanceBlock = async (provider: IProviders.WebSocketProvider) => {
  return provider.send('evm_mine', [])
}

// increases ganache time by the passed duration in seconds
export const increaseTime = async (provider: IProviders.WebSocketProvider, _duration: BigNumberish) => {
  const duration = ethers.BigNumber.from(_duration)

  if (duration.isNegative()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  await provider.send('evm_increaseTime', [duration.toNumber()])

  await advanceBlock(provider)
}

// advances to block
export const advanceBlockTo = async (provider: IProviders.WebSocketProvider, _target: BigNumberish) => {
  const target = ethers.BigNumber.from(_target)
  const currentBlock = await provider.getBlockNumber()
  const start = Date.now()
  let notified
  if (target.lt(currentBlock)) throw Error(`Target block #(${target}) is lower than current block #(${currentBlock})`)
  while (ethers.BigNumber.from(await provider.getBlockNumber()).lte(target)) {
    if (!notified && Date.now() - start >= 5000) {
      notified = true
    }
    await advanceBlock(provider)
  }
}
