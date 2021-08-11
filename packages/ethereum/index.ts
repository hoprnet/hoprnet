import type { Networks } from './constants'
import { join } from 'path'

export * from './constants'
export * from './types'
export * from './types/commons'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'

export type ContractData = {
  address: string
  transactionHash: string
  abi: any
}

export const getContractData = (
  network: Networks,
  contract: ContractNames,
  environmentId: string = 'default'
): ContractData => {
  // hack: required for E2E tests to pass
  // when a contract changes we redeploy it, this causes the deployments folder to change
  // unlike normal the release workflow, when running the E2E tests, we build the project
  // and then run deployments, which may update the deployment folder
  // this makes sure to always pick the deployment folder with the updated data
  const deploymentsPath = __dirname.endsWith('lib')
    ? join(__dirname, '..', 'deployments')
    : join(__dirname, 'deployments')

  try {
    return require(join(deploymentsPath, environmentId, network, `${contract}.json`))
  } catch {
    throw Error(`contract data for ${contract} from network ${network} not found`)
  }
}
