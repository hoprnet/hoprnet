import allAddresses from './addresses.json'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'
export type Networks = 'localhost' | 'mainnet' | 'kovan' | 'xdai' | 'matic' | 'binance'
export type DeploymentTypes = 'local' | 'staging' | 'production'

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
