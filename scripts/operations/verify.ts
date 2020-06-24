import { bash } from './utils'

export default async (network: string = 'development', ...contractNamesArr: string[]) => {
  const contractNames = contractNamesArr.join(' ')

  await bash(`npx truffle run verify ${contractNames} --network ${network}`)
}
