import { utils } from 'ethers'

export type PublicNetworks = 'xdai' | 'goerli' | 'mumbai' | 'polygon'
export type Networks = 'hardhat' | 'localhost' | PublicNetworks

/**
 * testing = for ganache / hardhat powered chains which do not auto mine
 * development = chains which automine - may or may not be public chains
 * staging = chain should be treated as production chain
 * production = our current production chain
 */
export type DeploymentTypes = 'testing' | 'development' | 'staging' | 'production'
export type NetworkTag = DeploymentTypes | 'etherscan'

export const networks: {
  [network in PublicNetworks]: {
    chainId: number
    gas?: number
  }
} = {
  xdai: {
    chainId: 100,
    gas: Number(utils.parseUnits('1', 'gwei'))
  },
  goerli: {
    chainId: 5
  },
  mumbai: {
    chainId: 80001,
    gas: Number(utils.parseUnits('1', 'gwei'))
  },
  polygon: {
    chainId: 137
  }
}
