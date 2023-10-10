import { type ChainOptions } from './index.js'

export const sampleChainOptions: ChainOptions = {
  chainId: 31337,
  network: 'anvil-localhost',
  maxFeePerGas: '10 gwei',
  maxPriorityFeePerGas: '1 gwei',
  chain: 'anvil',
  provider: 'http://localhost:8545',
  confirmations: 2
}
