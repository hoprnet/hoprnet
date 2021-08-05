import { utils } from 'ethers'

export type PublicNetworks = 'xdai' | 'goerli' | 'mumbai' | 'polygon'
export type Networks = 'hardhat' | 'localhost' | PublicNetworks
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
