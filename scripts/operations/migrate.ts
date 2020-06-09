import verify from './verify'
import { bash, isLocalNetwork, getContractNames } from './utils'

export default async (network: string = 'development') => {
  await bash(`npx truffle migrate --network ${network}`)

  // verification library currently doesn't work for solidity v6
  // if (!isLocalNetwork(network)) {
  //   await verify(network, ...getContractNames())
  // }
}
