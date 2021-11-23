export type Network = {
  id: string
  description: string
  chain_id: number // >= 0
  live: boolean
  default_provider: string // a valid HTTP url pointing at a RPC endpoint
  gas?: string // e.g. '1 gwei'
  gasPrice?: number // e.g. 1'
  gas_multiplier: number // e.g. 1.1
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
}

export type ProtocolConfig = {
  environments: Environment[]
  networks: Network[]
}

export type ResolvedEnvironment = {
  id: string
  network: Network
  channel_contract_deploy_block: number
  token_contract_address: string // an Ethereum address
  channels_contract_address: string // an Ethereum address
}

export function supportedEnvironments(): Environment[] {
  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  const environments = Object.entries(protocolConfig.environments).map(([id, env]) => ({ id, ...env }))
  return environments
}

export function resolveEnvironment(environment_id: string): ResolvedEnvironment {
  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  const environment = protocolConfig.environments[environment_id]
  const network = protocolConfig.networks[environment?.network_id]
  if (environment && network) {
    network.id = environment?.network_id
    return {
      id: environment_id,
      network,
      channel_contract_deploy_block: environment.channel_contract_deploy_block,
      token_contract_address: environment.token_contract_address,
      channels_contract_address: environment.channels_contract_address
    }
  }
  const supportedEnvsString: string = supportedEnvironments()
    .map((env) => env.id)
    .join(', ')
  throw new Error(`environment '${environment_id}' is not supported, supported environments: ${supportedEnvsString}`)
}
