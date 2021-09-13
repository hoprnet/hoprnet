export type Network = {
  id: string
  description: string
  chain_id: number // >= 0
  live: boolean
  default_provider: string // a valid HTTP url pointing at a RPC endpoint
  gas?: string // e.g. '1 gwei'
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

export function resolveEnvironment(environment_id: string): ResolvedEnvironment {
  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  for (const environment of protocolConfig.environments) {
    if (environment.id === environment_id) {
      for (const network of protocolConfig.networks) {
        if (network.id === environment.network_id) {
          return {
            id: environment.id,
            network,
            channel_contract_deploy_block: environment.channel_contract_deploy_block,
            token_contract_address: environment.token_contract_address,
            channels_contract_address: environment.channels_contract_address
          }
        }
      }
    }
  }
  const supportedEnvs: string = protocolConfig.environments.map((env) => env.id).join(', ')
  throw new Error(
    `failed to find environment with id '${environment_id}' in the supported protocol configuration, supported environments: ${supportedEnvs}`
  )
}