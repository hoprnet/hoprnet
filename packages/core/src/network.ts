import {
  resolve_network,
  supported_networks,
  type ChainOptions,
  type ResolvedNetwork,
  type Network
} from '../../hoprd/lib/hoprd_hoprd.js'

import type { DeploymentExtract } from '@hoprnet/hopr-core-ethereum'

export { resolve_network, supported_networks, type ChainOptions, type ResolvedNetwork, type Network }

export type EnvironmentType = 'production' | 'staging' | 'development' | 'local'

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
 * @param id network id, e.g. monte_rosa
 * @param customProvider
 * @returns the network details, throws if network is not supported
 */
export function resolveNetwork(id: string, customProvider?: string): ResolvedNetwork {
  return resolve_network(MONO_REPO_PATH, id, customProvider)
}

export function getContractData(id: string): DeploymentExtract {
  const resolvedNetwork = resolveNetwork(id)

  return {
    hoprAnnouncementsAddress: resolvedNetwork.announcements,
    hoprTokenAddress: resolvedNetwork.token,
    hoprChannelsAddress: resolvedNetwork.channels,
    hoprNetworkRegistryAddress: resolvedNetwork.network_registry,
    hoprNodeSafeRegistryAddress: resolvedNetwork.node_safe_registry,
    hoprTicketPriceOracleAddress: resolvedNetwork.ticket_price_oracle,
    indexerStartBlockNumber: resolvedNetwork.channel_contract_deploy_block
  }
}
