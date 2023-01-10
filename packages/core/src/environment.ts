import { resolve_environment, core_misc_set_panic_hook, supported_environments } from '../lib/core_misc.js'
import { DeploymentExtract } from '@hoprnet/hopr-core-ethereum/src/utils/utils.js'
core_misc_set_panic_hook()
export { resolve_environment, supported_environments } from '../lib/core_misc.js'

export type NetworkOptions = {
  id: string
  description: string
  chain_id: number // >= 0
  live: boolean
  default_provider: string // a valid HTTP url pointing at a RPC endpoint
  etherscan_api_url?: string // a valid HTTP url pointing at a RPC endpoint
  max_fee_per_gas: string // The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g. '10 gwei'
  max_priority_fee_per_gas: string // Tips paid directly to miners, e.g. '2 gwei'
  native_token_name: string
  hopr_token_name: string
  tags: string[]
}

export type EnvironmentType = 'production' | 'staging' | 'development'

export type Environment = {
  id: string
  network_id: string // must match one of the Network.id
  environment_type: EnvironmentType
  version_range: string
  channel_contract_deploy_block: number // >= 0
  token_contract_address: string // an Ethereum address
  channels_contract_address: string // an Ethereum address
  xhopr_contract_address: string // an Ethereum address,
  boost_contract_address: string // an Ethereum address,
  stake_contract_address: string // an Ethereum address,
  network_registry_proxy_contract_address: string // an Ethereum address,
  network_registry_contract_address: string // an Ethereum address,
}

export type ProtocolConfig = {
  environments: {
    [key: string]: Environment
  }
  networks: {
    [key: string]: NetworkOptions
  }
}

export type ResolvedEnvironment = {
  id: string
  network: NetworkOptions
  environment_type: EnvironmentType
  channel_contract_deploy_block: number
  token_contract_address: string // an Ethereum address
  channels_contract_address: string // an Ethereum address
  xhopr_contract_address: string // an Ethereum address,
  boost_contract_address: string // an Ethereum address,
  stake_contract_address: string // an Ethereum address,
  network_registry_proxy_contract_address: string // an Ethereum address,
  network_registry_contract_address: string // an Ethereum address,
}

const MONO_REPO_PATH = new URL('../../../', import.meta.url).pathname

/**
 * @param version HOPR version
 * @returns environments that the given HOPR version should be able to use
 */
export function supportedEnvironments(): Environment[] {
  return supported_environments(MONO_REPO_PATH)
}

/**
 * @param environment_id environment name
 * @param customProvider
 * @returns the environment details, throws if environment is not supported
 */
export function resolveEnvironment(environment_id: string, customProvider?: string): ResolvedEnvironment {
  return resolve_environment(MONO_REPO_PATH, environment_id, customProvider)
}

export const getContractData = (environment_id: string): DeploymentExtract => {
  const resolvedEnvironment = resolveEnvironment(environment_id)
  return {
    hoprTokenAddress: resolvedEnvironment.token_contract_address,
    hoprChannelsAddress: resolvedEnvironment.channels_contract_address,
    hoprNetworkRegistryAddress: resolvedEnvironment.network_registry_contract_address,
    indexerStartBlockNumber: resolvedEnvironment.channel_contract_deploy_block
  }
}
