import type { Network } from '../utils/networks'
import allAddresses from './addresses.json'

const addresses: {
  [network in Network]?: {
    // @TODO: dynamically type this
    [name in 'HoprToken' | 'HoprChannels' | 'HoprMinter' | 'HoprFaucet']?: string
  }
} = allAddresses

export default addresses
