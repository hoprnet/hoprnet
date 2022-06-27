import { type ChainOptions } from './index.js'

export const sampleChainOptions: ChainOptions = {
  chainId: 1337,
  environment: 'hardhat-localhost',
  maxFeePerGas: '10 gwei',
  maxPriorityFeePerGas: '1 gwei',
  network: 'hardhat',
  provider: 'http://localhost:8545'
}
