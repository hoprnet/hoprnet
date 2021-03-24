import { Network, addresses, abis } from '@hoprnet/hopr-ethereum'
import Web3 from 'web3'
import Debug from 'debug'
const log = Debug('hopr-core-ethereum:web3')

const HoprChannelsAbi = abis.HoprChannels
const HoprTokenAbi = abis.HoprToken

/**
 * Get current network's name.
 *
 * @param web3 a web3 instance
 * @returns the network's name
 */
export function getNetworkName(chainId: number): Network {
  switch (chainId) {
    case 1:
      return 'mainnet'
    // case 2:
    //   return 'morden'
    case 3:
      return 'ropsten'
    // case 4:
    //   return 'rinkeby'
    case 5:
      return 'goerli'
    case 42:
      return 'kovan'
    case 56:
      return 'binance'
    case 100:
      return 'xdai'
    case 137:
      return 'matic'
    default:
      return 'localhost'
  }
}

let initialized = false
let provider
let hoprChannels
let hoprToken
let web3
let network
let chainId
let address
export async function initialize(providerUri: string) {
  if (initialized) {
    return
  }
  provider = new Web3.providers.WebsocketProvider(providerUri, {
    reconnect: {
      auto: true,
      delay: 1000, // ms
      maxAttempts: 30
    }
  })

  provider.on('error', (e) => {
    log('web3 conn issue: ', e)
  })
  web3 = new Web3(provider)
  console.log('>>', providerUri)
  chainId = await web3.eth.getChainId()
  console.log('>>>', chainId)
  network = getNetworkName(chainId) as Network
  if (typeof addresses?.[network]?.HoprChannels === 'undefined') {
    throw Error(`token contract address from network ${network} not found`)
  }

  hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, addresses?.[network]?.HoprChannels)
  hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.[network]?.HoprToken)
  address = addresses[network]
  initialized = true
}

export function getWeb3() {
  if (!initialized) {
    throw new Error('Cannot access web3 before it is initialized')
  }

  return {
    provider,
    hoprChannels,
    hoprToken,
    web3,
    network,
    chainId,
    address
  }
}
