/*
  copied from OZ's text-helpers and modified to include a web3 param
  @TODO: find a way to re-use this through the original repo
*/
import { promisify } from 'util'
import BN from 'bn.js'
import Web3 from 'web3'

export function advanceBlock(web3: Web3) {
  // @ts-ignore
  return promisify(web3.currentProvider.send.bind(web3.currentProvider))({
    jsonrpc: '2.0',
    method: 'evm_mine',
    id: new Date().getTime()
  })
}

// Advance the block to the passed height
export async function advanceBlockTo(web3: Web3, target: BN | number) {
  if (!BN.isBN(target)) {
    target = new BN(target)
  }

  const currentBlock = await latestBlock(web3)

  if (target.lt(currentBlock)) throw Error(`Target block #(${target}) is lower than current block #(${currentBlock})`)
  while ((await latestBlock(web3)).lt(target)) {
    await advanceBlock(web3)
  }
}

// Returns the time of the last mined block in seconds
export async function latest(web3: Web3) {
  const block = await web3.eth.getBlock('latest')
  return new BN(block.timestamp)
}

export async function latestBlock(web3: Web3) {
  const block = await web3.eth.getBlock('latest')
  return new BN(block.number)
}

// Increases ganache time by the passed duration in seconds
export async function increase(web3: Web3, duration: BN | number) {
  if (!BN.isBN(duration)) {
    duration = new BN(duration)
  }

  if (duration.isNeg()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  // @ts-ignore
  await promisify(web3.currentProvider.send.bind(web3.currentProvider))({
    jsonrpc: '2.0',
    method: 'evm_increaseTime',
    params: [duration.toNumber()],
    id: new Date().getTime()
  })

  await advanceBlock(web3)
}

/**
 * Beware that due to the need of calling two separate ganache methods and rpc calls overhead
 * it's hard to increase time precisely to a target point so design your test to tolerate
 * small fluctuations from time to time.
 *
 * @param target time in seconds
 */
export async function increaseTo(web3: Web3, target: BN | number) {
  if (!BN.isBN(target)) {
    target = new BN(target)
  }

  const now = await latest(web3)

  if (target.lt(now)) throw Error(`Cannot increase current time (${now}) to a moment in the past (${target})`)
  const diff = target.sub(now)
  return increase(web3, diff)
}
