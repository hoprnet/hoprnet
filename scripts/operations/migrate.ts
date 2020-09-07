import verify from './verify'
import { bash } from './utils'
import networks from '../../truffle-networks'

export default async (network: string = 'development') => {
  await bash(`npx truffle migrate --network ${network}`)

  const config = networks[network]

  if (config.noVerify) return
  else if (config.network_type === 'mainnet') {
    await verify(network, 'HoprToken', 'HoprChannels', 'HoprMinter')
  } else if (config.network_type === 'testnet') {
    await verify(network, 'HoprToken', 'HoprChannels', 'HoprFaucet')
  }
}
