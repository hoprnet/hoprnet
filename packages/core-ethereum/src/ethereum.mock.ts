import { type ChainOptions } from './index.js'

export const sampleChainOptions: ChainOptions = {
  chainId: 31337,
  environment: 'anvil-localhost',
  maxFeePerGas: '10 gwei',
  maxPriorityFeePerGas: '1 gwei',
  network: 'anvil',
  provider: 'http://localhost:8545'
}
