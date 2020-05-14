import verify from './verify'
import { bash, isLocalNetwork, getContractNames } from './utils'

export default async (network: string = 'development') => {
  await bash(`npx truffle migrate --network ${network}`)

  if (!isLocalNetwork(network)) {
    await verify(network, ...getContractNames())
  }
}
