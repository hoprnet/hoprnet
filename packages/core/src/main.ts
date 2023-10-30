import { debug, Database, ChainKeypair, OffchainKeypair, HoprdConfig } from '@hoprnet/hopr-utils'

import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

import { Hopr } from './index.js'
import { DB_VERSION_TAG } from './constants.js'
import { getContractData, resolveNetwork } from './network.js'
import path from 'path'
import { rmSync } from 'fs'

const log = debug(`hopr-core:create-hopr`)

/*
 * General function to create a HOPR node given an identity and configuration
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param cfg:HoprdConfig - Required configuration to create node
 * @returns {Hopr} - HOPR node
 */
export async function createHoprNode(
  chainKeypair: ChainKeypair,
  packetKeypair: OffchainKeypair,
  cfg: HoprdConfig,
  automaticChainCreation = true
): Promise<Hopr> {
  const dbPath = path.join(cfg.db.data, 'db', DB_VERSION_TAG)
  if (cfg.db.force_initialize) {
    log(`force cleaning up existing database`)
    rmSync(dbPath, { recursive: true, force: true })
    cfg.db.initialize = true
  }
  let db = new Database(dbPath.toString(), cfg.db.initialize, chainKeypair.public().to_address())

  // if safe address or module address is not provided, replace with values stored in the db
  let safeAddress = cfg.safe_module.safe_address
  let moduleAddress = cfg.safe_module.module_address
  log(`options.safeModule.safeAddress: ${safeAddress}`)
  log(`options.safeModule.moduleAddress: ${moduleAddress}`)
  if (!safeAddress) {
    safeAddress = await db.get_staking_safe_address()
  }

  if (!moduleAddress) {
    moduleAddress = await db.get_staking_module_address()
  }

  if (!safeAddress || !moduleAddress) {
    log(`failed to provide safe or module address:`)
    throw new Error('Hopr Node must be initialized with safe and module address')
  }

  log(`using provider URL: ${cfg.chain.provider}`)

  const network = resolveNetwork(cfg.network, cfg.chain.provider)
  // get contract data for the given environment id and pass it on to create chain wrapper
  const resolvedContractAddresses = getContractData(network.id)
  log(`[DEBUG] resolvedContractAddresses ${network.id} ${JSON.stringify(resolvedContractAddresses, null, 2)}`)

  await HoprCoreEthereum.createInstance(
    db,
    chainKeypair,
    {
      chainId: network.chain.chain_id,
      network: network.id,
      maxFeePerGas: network.chain.max_fee_per_gas,
      maxPriorityFeePerGas: network.chain.max_priority_fee_per_gas,
      chain: network.chain.id,
      provider: network.chain.default_provider,
      confirmations: network.confirmations
    },
    {
      safeTransactionServiceProvider: cfg.safe_module.safe_transaction_service_provider,
      safeAddress,
      moduleAddress
    },
    resolvedContractAddresses,
    automaticChainCreation
  )

  // TODO: What is this?
  // // Initialize connection to the blockchain
  // await chain.initializeChainWrapper(resolvedContractAddresses)

  return new Hopr(chainKeypair, packetKeypair, db, cfg)
}
