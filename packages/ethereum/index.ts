import type { Networks } from './deploy/constants'

export * from './deploy/constants'
export * from './types'
export * from './types/commons'

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
