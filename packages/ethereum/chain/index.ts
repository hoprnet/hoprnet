import allAddresses from './addresses.json'

export type ContractNames = 'HoprToken' | 'HoprChannels'
export type Networks = 'localhost' | 'mainnet' | 'kovan' | 'xdai' | 'matic' | 'binance'

export const addresses: {
  [network in Networks]?: {
    [name in ContractNames]?: string
  }
} = allAddresses

export const abis: {
  [name in ContractNames]: any[]
} = {
  HoprToken: require('./abis/HoprToken.json'),
  HoprChannels: require('./abis/HoprChannels.json')
}
