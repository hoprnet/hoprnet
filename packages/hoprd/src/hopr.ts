import {
  debug,
  Database,
  ChainKeypair,
  OffchainKeypair,
  HoprLibConfig,
  Address,
  get_contract_data,
  resolve_network,
  WasmTxExecutor,
  Balance,
  retryWithBackoffThenThrow,
  durations,
  MIN_NATIVE_BALANCE,
  getBackoffRetryTimeout,
  getBackoffRetries,
  Hopr,
  HalfKeyChallenge,
  ApplicationData,
  TagBloomFilter,
  OffchainPublicKey,
  ChannelEntry,
  defer
} from '@hoprnet/hopr-utils'

import HoprCoreEthereum, {
  Indexer,
  ChannelUpdateEventNames,
  TicketRedeemedEventNames,
  PeerEventName,
  NetworkRegistryNodeAllowedEventName,
  NetworkRegistryNodeNotAllowedEventName,
  NetworkRegistryEligibilityChangedEventName
} from '@hoprnet/hopr-core-ethereum'

import EventEmitter from 'events'
import path from 'path'
import { rmSync, readFileSync, writeFileSync } from 'fs'
import { Multiaddr } from '@multiformats/multiaddr'
import { PeerId } from '@libp2p/interface-peer-id'

const log = debug(`hopr-lib:create-components`)

const DB_VERSION_TAG = 'main_4'

export class WasmChainQuery {
  private connector: HoprCoreEthereum = HoprCoreEthereum.getInstance()
  /**
   * Create a chain query node
   */
  public constructor(private me: Address) {}

  public clone(): WasmChainQuery {
    return new WasmChainQuery(this.me)
  }

  /*
   * Start the indexer and chain synchronization
   */
  public async startChainSync(): Promise<void> {
    await this.connector.start()
  }

  public async canRegisterWithSafe(): Promise<boolean> {
    return await this.connector.canRegisterWithSafe()
  }

  /**
   * DEPRECATED - HOPR balance is not used directly anymore
   *
   * we do not keep the node's hopr balance in the db anymore, therefore use the RPC API instead
   * TODO: remove these functions entirely since the HOPR balance isn't used anymore by the node
   *
   * @returns HOPR Balance
   */
  public async getBalance(): Promise<string> {
    return (await this.connector.getBalance(false)).to_string()
  }

  public async getNativeBalance(): Promise<string> {
    return (await this.connector.getNativeBalance(this.me.to_string(), true)).to_string()
  }

  public async getSafeBalance(): Promise<string> {
    return (await this.connector.getSafeBalance()).to_string()
  }

  public async getSafeNativeBalance(): Promise<string> {
    return (await this.connector.getNativeBalance(this.smartContractInfo().safeAddress, true)).to_string()
  }

  public indexer(): Indexer {
    return this.connector.indexer
  }

  public async getPacketKeyOf(address: Address) {
    return this.connector.getPacketKeyOf(address)
  }

  public async isNodeSafeNotRegistered(): Promise<boolean> {
    return await this.connector.isNodeSafeNotRegistered()
  }

  /**
   * This is a utility method to wait until the node is funded.
   * A backoff is implemented, if node has not been funded and
   * MAX_DELAY is reached, this function will reject.
   */
  public async waitForFunds(): Promise<void> {
    const minDelay = durations.seconds(1)
    const maxDelay = durations.seconds(200)
    const delayMultiple = 1.05
    try {
      await retryWithBackoffThenThrow(
        () =>
          new Promise<void>(async (resolve, reject) => {
            try {
              // call connector directly and don't use cache, since this is
              // most likely outdated during node startup
              const nativeBalance = await this.connector.getNativeBalance(this.me.to_string())
              log(`waitforfunds: current balance is ${nativeBalance.to_formatted_string()}`)

              if (nativeBalance.gte(nativeBalance.of_same(MIN_NATIVE_BALANCE.toString(10)))) {
                resolve()
              } else {
                log('waitforfunds: still unfunded, trying again soon')
                reject()
              }
            } catch (e) {
              log('waitforfunds: error with native balance call, trying again soon')
              reject()
            }
          }),
        {
          minDelay,
          maxDelay,
          delayMultiple
        }
      )
    } catch {
      log(
        `unfunded for more than ${getBackoffRetryTimeout(
          minDelay,
          maxDelay,
          delayMultiple
        )} seconds and ${getBackoffRetries(minDelay, maxDelay, delayMultiple)} retries, shutting down`
      )
      await HoprCoreEthereum.getInstance().stop()
      process.exit(1)
    }
  }

  public smartContractInfo(): {
    chain: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    hoprNetworkRegistryAddress: string
    hoprNodeSafeRegistryAddress: string
    hoprTicketPriceOracleAddress: string
    moduleAddress: string
    safeAddress: string
    noticePeriodChannelClosure: number
  } {
    return this.connector.smartContractInfo()
  }

  public on(event: string, callback: () => void): void {
    this.connector.on(event, callback)
  }
  public emit(event: string): void {
    this.connector.emit(event)
  }
}

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
  let db_for_ethereum = db.clone()

  // CHAIN ========
  log(`Creating chain components using provider URL: ${cfg.chain.provider}`)
  const chain_config = resolve_network(cfg.chain.network, cfg.chain.provider)
  const resolvedContractAddresses = get_contract_data(cfg.chain.network, cfg.chain.provider)
  log(`[DEBUG] resolvedContractAddresses ${chain_config.id} ${JSON.stringify(resolvedContractAddresses, null, 2)}`)

  await HoprCoreEthereum.createInstance(
    db_for_ethereum,
    chainKeypair,
    {
      chainId: chain_config.chain.chain_id,
      network: chain_config.id,
      maxFeePerGas: chain_config.chain.max_fee_per_gas,
      maxPriorityFeePerGas: chain_config.chain.max_priority_fee_per_gas,
      chain: chain_config.chain.id,
      provider: chain_config.chain.default_provider,
      confirmations: chain_config.confirmations
    },
    {
      safeTransactionServiceProvider: cfg.safe_module.safe_transaction_service_provider,
      safeAddress: cfg.safe_module.safe_address,
      moduleAddress: cfg.safe_module.module_address
    },
    resolvedContractAddresses,
    false
  )
  
  let chain_query = new WasmChainQuery(chainKeypair.public().to_address())
  
  let connector = HoprCoreEthereum.getInstance()

  log(`${chainKeypair.public().to_hex(false)}: ${resolvedContractAddresses.hopr_channels_address},
    ${resolvedContractAddresses.hopr_announcements_address}, ${resolvedContractAddresses.hopr_announcements_address},
    ${cfg.safe_module.module_address}, ${resolvedContractAddresses.hopr_node_safe_registry_address}, ${resolvedContractAddresses.hopr_token_address}`)
  let tx_executor = new WasmTxExecutor(
    connector.sendTransaction.bind(connector),
    chainKeypair,
    Address.from_string(resolvedContractAddresses.hopr_channels_address),
    Address.from_string(resolvedContractAddresses.hopr_announcements_address),
    cfg.safe_module.module_address,
    Address.from_string(resolvedContractAddresses.hopr_node_safe_registry_address),
    Address.from_string(resolvedContractAddresses.hopr_token_address)
  )

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
    tx_executor,
    message_emitter as WasmHoprMessageEmitter,
    chain_query as WasmChainQuery,
    onReceivedMessage,
    onAcknowledgement
  )

  log('Setting up the indexer events...')
  let hc = HoprCoreEthereum.getInstance()
  let indexer = hc.indexer
  indexer.on(NetworkRegistryEligibilityChangedEventName, async (address: Address, allowed: boolean) => {
    // If account is no longer eligible to register nodes, we might need to close existing connections,
    // otherwise there is nothing to do
    if (!allowed) {
      let pk: OffchainPublicKey
      try {
        pk = await hc.getPacketKeyOf(address)
      } catch (err) {
        // node has not announced itself, so we don't need to care
        return
      }

      const peer = pk.to_peerid_str()
      await hopr.unregister(peer)
    }
  })

  indexer.on(PeerEventName, async (peerData: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }) => {
    const peer = peerData.id.toString()
    const address = peerData.address
    const mas = peerData.multiaddrs.map((ma) => ma.toString())
    await hopr.onPeerAnnouncement(peer, address, mas)
  })

  indexer.on(NetworkRegistryNodeAllowedEventName, async (node: Address) => {
    await hopr.onNetworkRegistryUpdate(node, true)
  })

  indexer.on(NetworkRegistryNodeNotAllowedEventName, async (node: Address) => {
    await hopr.onNetworkRegistryUpdate(node, false)
  })

  // subscribe so we can process channel close events
  indexer.on(ChannelUpdateEventNames, async (channel: ChannelEntry) => {
    await hopr.onOwnChannelUpdated(channel)
  })

  // subscribe so we can process channel ticket redeemed events
  indexer.on(TicketRedeemedEventNames, async (channel: ChannelEntry, ticketAmount: Balance) => {
    await hopr.onTicketRedeemed(channel, ticketAmount)
  })

  let continueStartup = defer<void>()
  hc.on('hopr:connector:created', () => {
    // 2.b - Connector has been created, and we can now trigger the next set of steps.
    log('Connector has been loaded properly.')
    continueStartup.resolve()
  })

  // 2.a - Setup connector listener to bubble up to node. Emit connector creation.
  log(`Ready to request on-chain connector to connect to provider.`)
  hc.emit('connector:create')

  await continueStartup.promise

  return hopr
}
