import allAddresses from './addresses.json'
import { Networks } from './networks'

export * from './networks'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'

export const addresses: {
  [network in Networks]?: {
    [name in ContractNames]?: string
  }
} = allAddresses

export const abis: {
  [name in ContractNames]: any[]
} = {
  HoprToken: require('./abis/HoprToken.json'),
  HoprChannels: require('./abis/HoprChannels.json'),
  HoprDistributor: require('./abis/HoprDistributor.json')
}
