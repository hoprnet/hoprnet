import { ethers, providers } from 'ethers'

export const advanceBlock = async (provider: providers.StaticJsonRpcProvider) => {
  return provider.send('evm_mine', [])
}

// increases ganache time by the passed duration in seconds
export const increaseTime = async (provider: providers.StaticJsonRpcProvider, _duration: ethers.BigNumberish) => {
  const duration = ethers.BigNumber.from(_duration)

  if (duration.isNegative()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  await provider.send('evm_increaseTime', [duration.toNumber()])

  await advanceBlock(provider)
}

export const latestBlock = async (provider: providers.StaticJsonRpcProvider) => {
  return provider.getBlockNumber()
}

export const latestBlockTime = async (provider: providers.StaticJsonRpcProvider): Promise<number> => {
  const latest = await latestBlock(provider)
  const block = await provider.getBlock(latest)
  return block.timestamp
}

export const advanceTimeForNextBlock = async (
  provider: providers.StaticJsonRpcProvider,
  blockTimestampInSec: number
) => {
  await provider.send('evm_setNextBlockTimestamp', [blockTimestampInSec])
  await provider.send('evm_mine', [blockTimestampInSec])
}
