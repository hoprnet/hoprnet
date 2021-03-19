import Web3 from 'web3'

export type PublicNetworks = 'mainnet' | 'kovan' | 'xdai' | 'matic' | 'binance'
export type Networks = 'hardhat' | 'localhost' | PublicNetworks
export type DeploymentTypes = 'local' | 'staging' | 'production'

export const networks: {
  [network in PublicNetworks]: {
    live: boolean
    chainId: number
    tags?: string[]
    gas?: number
  }
} = {
  mainnet: {
    live: true,
    tags: ['production', 'etherscan'],
    chainId: 1
  },
  kovan: {
    live: true,
    tags: ['staging', 'etherscan'],
    chainId: 42,
    gas: Number(Web3.utils.toWei('1', 'gwei'))
  },
  xdai: {
    live: true,
    tags: ['staging'],
    chainId: 100,
    gas: Number(Web3.utils.toWei('1', 'gwei'))
  },
  matic: {
    live: true,
    tags: ['staging'],
    chainId: 137,
    gas: Number(Web3.utils.toWei('1', 'gwei'))
  },
  binance: {
    live: true,
    tags: ['staging'],
    chainId: 56,
    gas: Number(Web3.utils.toWei('20', 'gwei')) // binance chain requires >= 20gwei
  }
}
