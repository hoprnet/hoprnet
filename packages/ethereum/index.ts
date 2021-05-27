import type { Networks } from './constants'
import { join } from 'path'

export * from './constants'
export * from './types'
export * from './types/commons'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'

export type ContractData = {
  address: string
  transactionHash: string
  abi: any
}

export const getContractData = (network: Networks, contract: ContractNames): ContractData => {
  const deploymentsPath = __dirname === 'lib' ? join(__dirname, '..', 'deployments') : join(__dirname, 'deployments')

  try {
    return require(join(deploymentsPath, network, `${contract}.json`))
  } catch {
    throw Error(`contract data for ${contract} from network ${network} not found`)
  }
}
