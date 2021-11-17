import { join } from 'path'

export * from './constants'
export * from './types'
export type { TypedEventFilter, TypedEvent } from './types/common'

export type ContractNames = 'HoprToken' | 'HoprChannels' | 'HoprDistributor'

export type ContractData = {
  address: string
  transactionHash: string
  abi: any
}

export const getContractData = (network: string, environmentId: string, contract: ContractNames): ContractData => {
  // hack: required for E2E tests to pass
  // when a contract changes we redeploy it, this causes the deployments folder to change
  // unlike normal the release workflow, when running the E2E tests, we build the project
  // and then run deployments, which may update the deployment folder
  // this makes sure to always pick the deployment folder with the updated data
  const deploymentsPath = join(__dirname, '..', 'deployments', environmentId, network, `${contract}.json`)

  try {
    return require(deploymentsPath)
  } catch {
    throw Error(`contract data for ${contract} from environment ${environmentId} and network ${network} not found`)
  }
}
