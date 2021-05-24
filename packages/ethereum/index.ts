import type { Networks } from './deploy/constants'

export type { HoprToken, HoprChannels } from './types'
export type { TypedEvent, TypedEventFilter } from './types/commons'

export * from './deploy/constants'
export { HoprToken__factory, HoprChannels__factory } from './types'

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
