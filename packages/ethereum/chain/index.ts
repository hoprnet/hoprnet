import type { Network } from '../utils/networks'
import allAddresses from './addresses.json'

// @TODO: dynamically type this
export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprMinter' | 'HoprFaucet'

export const addresses: {
  [network in Network]?: {
    [name in ContractNames]?: string
  }
} = allAddresses

export const abis: {
  [name in ContractNames]: any[]
} = {
  HoprToken: require('./abis/HoprToken.json'),
  HoprChannels: require('./abis/HoprChannels.json'),
  HoprMinter: require('./abis/HoprMinter.json'),
  HoprFaucet: require('./abis/HoprFauce.json')
}
