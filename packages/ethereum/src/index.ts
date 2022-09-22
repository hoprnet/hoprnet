import { join } from 'path'
import { Deployment } from 'hardhat-deploy/dist/types'

export * from './constants'
export type {
  HoprToken,
  HoprChannels,
  HoprDistributor,
  HoprNetworkRegistry,
  HoprBoost,
  HoprStake,
  HoprStake2,
  HoprStakeSeason3,
  HoprStakeSeason4,
  HoprWhitehat,
  // used by libraries that want to interact with xHOPR
  ERC677 as xHoprToken
} from './types'
export type { TypedEventFilter, TypedEvent } from './types/common'

export type ContractNames =
  | 'HoprToken'
  | 'HoprChannels'
  | 'HoprDistributor'
  | 'HoprNetworkRegistry'
  | 'HoprBoost'
  | 'HoprStake'
  | 'HoprStake2'
  | 'HoprStakeSeason3'
  | 'HoprStakeSeason4'
  | 'HoprWhitehat'

export type ContractData = {
  address: string
  transactionHash: string
  abi: any
  blockNumber: number
}

export const getContractData = (
  network: string,
  environmentId: string,
  contract: ContractNames
): Deployment | ContractData => {
  // hack: required for E2E tests to pass
  // when a contract changes we redeploy it, this causes the deployments folder to change
  // unlike normal the release workflow, when running the E2E tests, we build the project
  // and then run deployments, which may update the deployment folder
  // this makes sure to always pick the deployment folder with the updated data
  const deploymentsPath = join(
    '..',
    'deployments',
    environmentId,
    network === 'hardhat' ? 'localhost' : network,
    `${contract}.json`
  )

  try {
    return require(deploymentsPath)
  } catch {
    throw Error(
      `contract data for ${contract} from environment ${environmentId} and network ${network} not found in ${deploymentsPath}`
    )
  }
}
