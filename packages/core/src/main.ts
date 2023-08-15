import path from 'path'

import type { PeerId } from '@libp2p/interface-peer-id'
import { keysPBM } from '@libp2p/crypto/keys'

import { PublicKey, debug, LevelDb } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

import Hopr, { type HoprOptions } from './index.js'
import { getContractData } from './network.js'
import { Database, PublicKey as Database_PublicKey, core_hopr_initialize_crate } from '../lib/core_hopr.js'
core_hopr_initialize_crate()

const log = debug(`hopr-core:create-hopr`)

/*
 * General function to create a HOPR node given an identity an
 * range of options.
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @returns {Hopr} - HOPR node
 */
export async function createHoprNode(
  peerId: PeerId,
  options: HoprOptions,
  automaticChainCreation = true
): Promise<Hopr> {
  let levelDb = new LevelDb()

  try {
    const dbPath = path.join(options.dataPath, 'db')
    await levelDb.init(options.createDbIfNotExist, dbPath, options.forceCreateDB, options.network.id)

    // Dump entire database to a file if given by the env variable
    const dump_file = process.env.DB_DUMP ?? ''
    if (dump_file.length > 0) {
      await levelDb.dump(dump_file)
    }
  } catch (err: unknown) {
    log(`failed init db:`, err)
    throw err
  }

  let db = new Database(levelDb, Database_PublicKey.from_peerid_str(peerId.toString()).to_address())

  log(`using provider URL: ${options.network.chain.default_provider}`)
  const chain = HoprCoreEthereum.createInstance(
    db,
    PublicKey.from_peerid_str(peerId.toString()),
    keysPBM.PrivateKey.decode(peerId.privateKey as Uint8Array).Data,
    {
      chainId: options.network.chain.chain_id,
      network: options.network.id,
      maxFeePerGas: options.network.chain.max_fee_per_gas,
      maxPriorityFeePerGas: options.network.chain.max_priority_fee_per_gas,
      chain: options.network.chain.id,
      provider: options.network.chain.default_provider
    },
    automaticChainCreation
  )

  // get contract data for the given environment id and pass it on to create chain wrapper
  const resolvedContractAddresses = getContractData(options.network.id)
  log(`[DEBUG] resolvedContractAddresses ${options.network.id} ${JSON.stringify(resolvedContractAddresses, null, 2)}`)
  // Initialize connection to the blockchain
  await chain.initializeChainWrapper(resolvedContractAddresses)

  return new Hopr(peerId, db, options)
}
