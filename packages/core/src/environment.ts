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

export type Environment = {
  id: string
  network_id: string // must match one of the Network.id
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
  environments: Environment[]
  networks: NetworkOptions[]
}

export type ResolvedEnvironment = {
  id: string
  network: NetworkOptions
  channel_contract_deploy_block: number
  token_contract_address: string // an Ethereum address
  channels_contract_address: string // an Ethereum address
  xhopr_contract_address: string // an Ethereum address,
  boost_contract_address: string // an Ethereum address,
  stake_contract_address: string // an Ethereum address,
  network_registry_proxy_contract_address: string // an Ethereum address,
  network_registry_contract_address: string // an Ethereum address,
}

export function supportedEnvironments(): Environment[] {
  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  const environments = Object.entries(protocolConfig.environments).map(([id, env]) => ({ id, ...env }))
  return environments
}

export function resolveEnvironment(environment_id: string, customProvider?: string): ResolvedEnvironment {
  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  const environment = protocolConfig.environments[environment_id]
  const network = protocolConfig.networks[environment?.network_id]
  if (environment && network) {
    network.id = environment?.network_id
    network.default_provider = customProvider ?? network?.default_provider
    return {
      id: environment_id,
      network,
      channel_contract_deploy_block: environment.channel_contract_deploy_block,
      token_contract_address: environment.token_contract_address,
      channels_contract_address: environment.channels_contract_address,
      xhopr_contract_address: environment.xhopr_contract_address,
      boost_contract_address: environment.boost_contract_address,
      stake_contract_address: environment.stake_contract_address,
      network_registry_proxy_contract_address: environment.network_registry_proxy_contract_address,
      network_registry_contract_address: environment.network_registry_contract_address
    }
  }
  const supportedEnvsString: string = supportedEnvironments()
    .map((env) => env.id)
    .join(', ')
  throw new Error(`environment '${environment_id}' is not supported, supported environments: ${supportedEnvsString}`)
}
