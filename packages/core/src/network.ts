import {
  resolve_network,
  core_misc_set_panic_hook,
  supported_networks,
  type ChainOptions,
  type ResolvedNetwork,
  type Network
} from '../lib/core_misc.js'
import { DeploymentExtract } from '@hoprnet/hopr-core-ethereum/src/utils/utils.js'
core_misc_set_panic_hook()
export {
  resolve_network,
  supported_networks,
  type ChainOptions,
  type ResolvedNetwork,
  type Network
} from '../lib/core_misc.js'

export type EnvironmentType = 'production' | 'staging' | 'development'

export type ProtocolConfig = {
  networks: {
    [key: string]: Network
  }
  chains: {
    [key: string]: ChainOptions
  }
}

const MONO_REPO_PATH = new URL('../../../', import.meta.url).pathname

/**
 * @param version HOPR version
 * @returns environments that the given HOPR version should be able to use
 */
export function supportedNetworks(): Network[] {
  return supported_networks(MONO_REPO_PATH)
}

/**
 * @param network_id network id, e.g. monte_rosa
 * @param customProvider
 * @returns the network details, throws if network is not supported
 */
export function resolveNetwork(network_id: string, customProvider?: string): ResolvedNetwork {
  return resolve_network(MONO_REPO_PATH, network_id, customProvider)
}

export const getContractData = (network_id: string): DeploymentExtract => {
  const resolvedNetwork = resolveNetwork(network_id)
  return {
    hoprTokenAddress: resolvedNetwork.token_contract_address,
    hoprChannelsAddress: resolvedNetwork.channels_contract_address,
    hoprNetworkRegistryAddress: resolvedNetwork.network_registry_contract_address,
    indexerStartBlockNumber: resolvedNetwork.channel_contract_deploy_block
  }
}
