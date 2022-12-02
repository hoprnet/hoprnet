// May not change at runtime
// Don't do type-checks on JSON files
// @ts-ignore
import protocolConfig from '../protocol-config.json' assert { type: 'json' }
import semver from 'semver'
import { FULL_VERSION } from './constants.js'

/**
 * Coerced full version using
 * semver.coerce('42.6.7.9.3-alpha') // '42.6.7'
 */
const FULL_VERSION_COERCED = semver.coerce(FULL_VERSION).version

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

/**
 * @param version HOPR version
 * @returns environments that the given HOPR version should be able to use
 */
export function supportedEnvironments(): Environment[] {
  const environments = Object.entries((protocolConfig as ProtocolConfig).environments)

  return environments
    .filter(([_, env]) => {
      return semver.satisfies(FULL_VERSION_COERCED, env.version_range)
    })
    .map(([id, env]) => ({
      id,
      ...env
    }))
}

/**
 * @param environment_id environment name
 * @param customProvider
 * @returns the environment details, throws if environment is not supported
 */
export function resolveEnvironment(environment_id: string, customProvider?: string): ResolvedEnvironment {
  console.log(`[DEBUG] resolveEnvironment ${environment_id}, ${customProvider}`);
  const environment = (protocolConfig as ProtocolConfig).environments[environment_id]
  console.log(`[DEBUG] resolveEnvironmentenvironment environment ${environment}`);
  const network = (protocolConfig as ProtocolConfig).networks[environment?.network_id]
  console.log(`[DEBUG] resolveEnvironmentenvironment network ${network}`);
  console.log(`[DEBUG] resolveEnvironmentenvironment semver.satisfies ${semver.satisfies(FULL_VERSION_COERCED, environment.version_range)}`);

  if (environment && network && semver.satisfies(FULL_VERSION_COERCED, environment.version_range)) {
    network.id = environment.network_id
    if (customProvider && customProvider.length > 0) {
      network.default_provider = customProvider
    }

    const debug = {
      id: environment_id,
      network,
      environment_type: environment.environment_type,
      channel_contract_deploy_block: environment.channel_contract_deploy_block,
      token_contract_address: environment.token_contract_address,
      channels_contract_address: environment.channels_contract_address,
      xhopr_contract_address: environment.xhopr_contract_address,
      boost_contract_address: environment.boost_contract_address,
      stake_contract_address: environment.stake_contract_address,
      network_registry_proxy_contract_address: environment.network_registry_proxy_contract_address,
      network_registry_contract_address: environment.network_registry_contract_address
    }
    console.log(`[DEBUG] return ${JSON.stringify(debug, null, 2)}`);
    return debug;
  }

  const supportedEnvsString: string = supportedEnvironments()
    .map((env) => env.id)
    .join(', ')
  throw new Error(`environment '${environment_id}' is not supported, supported environments: ${supportedEnvsString}`)
}
