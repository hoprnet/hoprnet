import { bash } from './utils'

export default async (network: string = 'development') => {
  await bash(`npx truffle migrate --network ${network}`)
}
