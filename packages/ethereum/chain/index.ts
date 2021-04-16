import allContracts from './contracts.json'
import { Networks } from './networks'

export * from './networks'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'
export type ContractData = {
  address: string
  deployedAt?: number
}

// TODO: this doesn't have to be a funciton
// change once 'core-ethereum' is refactored
export const getContracts = (): {
  [network in Networks]?: {
    [name in ContractNames]?: ContractData
  }
} => allContracts

export const abis: {
  [name in ContractNames]: any[]
} = {
  HoprToken: require('./abis/HoprToken.json'),
  HoprChannels: require('./abis/HoprChannels.json'),
  HoprDistributor: require('./abis/HoprDistributor.json')
}
