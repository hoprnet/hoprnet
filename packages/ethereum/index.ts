import { utils } from 'ethers'

export type PublicNetworks = 'xdai' | 'goerli'
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
  }
}

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'

export type ContractData = {
  address: string
  transactionHash: string
  abi: any
}

export const getContractData = (network: Networks, contract: ContractNames): ContractData => {
  try {
    return require(`./deployments/${network}/${contract}.json`)
  } catch {
    throw Error(`contract data for ${contract} from network ${network} not found`)
  }
}
