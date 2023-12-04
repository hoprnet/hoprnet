import {
  debug,
  Database,
  ChainKeypair,
  OffchainKeypair,
  HoprLibConfig,
  get_contract_data,
  resolve_network,
  Hopr,
  HalfKeyChallenge,
  ApplicationData,
  TagBloomFilter
} from '@hoprnet/hopr-utils'

import EventEmitter from 'events'
import path from 'path'
import { rmSync, readFileSync, writeFileSync } from 'fs'

const log = debug(`hopr-lib:create-components`)

const DB_VERSION_TAG = 'main_4'

/*
 * General HoprMessageEmitter object responsible for emitting
 * data that would otherwise be obtained from the core-transport
 * stream.
 *
 * Ingress part of the chain.
 */
export class WasmHoprMessageEmitter extends EventEmitter {
  public constructor() {
    super()
  }

  public delegate_on(event: string, callback: () => void) {
    this.on(event, callback)
  }
}

/*
 * General function to create a HOPR node given an identity and configuration
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param cfg:HoprdConfig - Required configuration to create node
 * @returns {Hopr} - HOPR node
 */
export async function createHoprNode(
  chainKeypair: ChainKeypair,
  packetKeypair: OffchainKeypair,
  cfg: HoprLibConfig
): Promise<Hopr> {
  // pre-flight checks
  if (!cfg.chain.announce) {
    throw new Error('Announce option should be turned ON in Providence, only public nodes are supported')
  }

  const dbPath = path.join(cfg.db.data, 'db', DB_VERSION_TAG)

  // COMMON ======
  // DB still needed for the chain components that use it, hence it has to be created here
  if (cfg.db.force_initialize) {
    log(`force cleaning up existing database`)
    rmSync(dbPath, { recursive: true, force: true })
    cfg.db.initialize = true
  }

  log(`Creating hopr-lib database in ${dbPath.toString()}: ${cfg.db.initialize}`)
  let db = new Database(dbPath.toString(), cfg.db.initialize, chainKeypair.public().to_address())

  // CHAIN ========
  log(`Creating chain components using provider URL: ${cfg.chain.provider}`)
  const chain_config = resolve_network(cfg.chain.network, cfg.chain.provider)
  const resolvedContractAddresses = get_contract_data(cfg.chain.network, cfg.chain.provider)
  log(`[DEBUG] resolvedContractAddresses ${chain_config.id} ${JSON.stringify(resolvedContractAddresses, null, 2)}`)

  log(`${chainKeypair.public().to_hex(false)}: ${resolvedContractAddresses.hopr_channels_address},
    ${resolvedContractAddresses.hopr_announcements_address}, ${resolvedContractAddresses.hopr_announcements_address},
    ${cfg.safe_module.module_address.to_hex()}, ${resolvedContractAddresses.hopr_node_safe_registry_address}, ${
    resolvedContractAddresses.hopr_token_address
  }`)

  // NODE Rust to TS hacked setup before fully migrating everything
  let message_emitter = new WasmHoprMessageEmitter()
  const onAcknowledgement = (ackChallenge: HalfKeyChallenge) => {
    // Can subscribe to both: per specific message or all message acknowledgments
    message_emitter.emit(`hopr:message-acknowledged:${ackChallenge.to_hex()}`)
    message_emitter.emit('hopr:message-acknowledged', ackChallenge.to_hex())
  }

  const onReceivedMessage = (msg: ApplicationData) => {
    message_emitter.emit('hopr:message', msg)
  }

  const tbfPath = path.join(cfg.db.data, 'tbf')
  log(`Initialize the Bloom filter storage in '${tbfPath}'`)
  let tagBloomFilter = new TagBloomFilter()
  try {
    let tbfData = new Uint8Array(readFileSync(tbfPath))
    tagBloomFilter = TagBloomFilter.deserialize(tbfData)
  } catch (err) {
    log(`no tag bloom filter file found, using empty`)
  }

  const storeTagBloomFilterContent = (tbfData: Uint8Array) => {
    try {
      writeFileSync(tbfPath, tbfData)
    } catch (err) {
      log(`failed to save tag bloom filter data`)
    }
  }

  log('Creating the hopr-lib node instance...')
  let hopr = new Hopr(
    cfg,
    packetKeypair,
    chainKeypair,
    db,
    tagBloomFilter,
    storeTagBloomFilterContent,
    message_emitter as WasmHoprMessageEmitter,
    onReceivedMessage,
    onAcknowledgement
  )

  return hopr
}
