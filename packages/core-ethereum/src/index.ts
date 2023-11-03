import type { PeerId } from '@libp2p/interface-peer-id'
import { ChainWrapper, createChainWrapper, SendTransactionStatus } from './ethereum.js'
import { BigNumber } from 'ethers'
import {
  AccountEntry,
  Address,
  Balance,
  BalanceType,
  cacheNoArgAsyncFunction,
  ChainKeypair,
  ChannelEntry,
  CORE_ETHEREUM_CONSTANTS,
  Database,
  debug,
  type DeferType,
  Hash,
  OffchainPublicKey,
  PublicKey,
  WasmTransactionPayload,
  SmartContractConfig
} from '@hoprnet/hopr-utils'

import { TransactionPayload } from './transaction-manager.js'

import Indexer from './indexer/index.js'
import { EventEmitter } from 'events'
import type { IndexerEventsNames, IndexerEventsType } from './indexer/types.js'

export {
  BlockEventName,
  BlockProcessedEventName,
  StatusEventName,
  PeerEventName,
  NetworkRegistryEligibilityChangedEventName,
  NetworkRegistryStatusChangedEventName,
  NetworkRegistryNodeAllowedEventName,
  NetworkRegistryNodeNotAllowedEventName,
  ChannelUpdateEventNames,
  TicketRedeemedEventNames
} from './indexer/types.js'

const log = debug('hopr-core-ethereum')

const MONO_REPO_PATH = new URL('../../../', import.meta.url).pathname

export function get_monorepo_path(): string {
  return MONO_REPO_PATH
}

export type ChainOptions = {
  provider: string
  confirmations: number
  chainId: number
  maxFeePerGas: string
  maxPriorityFeePerGas: string
  chain: string
  network: string
}

export type SafeModuleOptions = {
  safeTransactionServiceProvider?: string
  safeAddress: Address
  moduleAddress: Address
}

// Exported from Rust
const constants = CORE_ETHEREUM_CONSTANTS()

export default class HoprCoreEthereum extends EventEmitter {
  private static _instance: HoprCoreEthereum

  public indexer: Indexer
  private chain: ChainWrapper
  private started: boolean
  // Used to store ongoing operations to prevent duplicate redemption attempts

  private constructor(
    private db: Database,
    private chainKeypair: ChainKeypair,
    private options: ChainOptions,
    private safeModuleOptions: SafeModuleOptions,
    private automaticChainCreation: boolean
  ) {
    super()

    log(`[DEBUG] initialized Rust DB... ${JSON.stringify(this.db.toString(), null, 2)} `)

    this.indexer = new Indexer(
      this.chainKeypair.public().to_address(),
      this.db,
      this.options.confirmations,
      constants.INDEXER_BLOCK_RANGE
    )
  }

  public static async createInstance(
    db: Database,
    chainKeypair: ChainKeypair,
    options: ChainOptions,
    safeModuleOptions: SafeModuleOptions,
    deploymentAddresses: SmartContractConfig,
    automaticChainCreation = true
  ) {
    HoprCoreEthereum._instance = new HoprCoreEthereum(
      db,
      chainKeypair,
      options,
      safeModuleOptions,
      automaticChainCreation
    )
    // Initialize connection to the blockchain
    await HoprCoreEthereum._instance.initializeChainWrapper(deploymentAddresses)
    return HoprCoreEthereum._instance
  }

  public static getInstance(): HoprCoreEthereum {
    if (!HoprCoreEthereum._instance) throw new Error('non-existent instance of HoprCoreEthereum')
    return HoprCoreEthereum._instance
  }

  async initializeChainWrapper(deploymentAddresses: SmartContractConfig) {
    // In some cases, we want to make sure the chain within the connector is not triggered
    // automatically but instead via an event. This is the case for `hoprd`, where we need
    // to get notified after ther chain was properly created, and we can't get setup the
    // listeners before the node was actually created.
    log(`[DEBUG] initializeChainWrapper... ${JSON.stringify(deploymentAddresses, null, 2)} `)
    if (this.automaticChainCreation) {
      await this.createChain(deploymentAddresses)
    } else {
      this.once('connector:create', this.createChain.bind(this, deploymentAddresses))
    }
  }

  private async createChain(deploymentAddresses: SmartContractConfig): Promise<void> {
    try {
      log(
        `[DEBUG] createChain createChainWrapper starting with deploymentAddresses... ${JSON.stringify(
          deploymentAddresses,
          null,
          2
        )} `
      )
      log(
        `[DEBUG] createChain createChainWrapper starting with safeModuleOptions... ${JSON.stringify(
          this.safeModuleOptions,
          null,
          2
        )} `
      )
      this.chain = await createChainWrapper(
        deploymentAddresses,
        this.safeModuleOptions,
        this.options,
        this.chainKeypair
      )
    } catch (err) {
      const errMsg = 'failed to create provider chain wrapper'
      log(`error: ${errMsg}`, err)
      throw Error(errMsg)
    }

    // Emit event to make sure connector is aware the chain was created properly.
    this.emit('hopr:connector:created')
  }

  async start() {
    if (!this.started) {
      try {
        await this.chain.waitUntilReady()

        // indexer starts
        await this.indexer.start(this.chain, this.chain.getGenesisBlock(), this.safeModuleOptions.safeAddress)

        // Debug log used in e2e integration tests, please don't change
        log(`using blockchain address ${this.chainKeypair.to_address().to_hex()}`)
        log('Connector started')
        this.started = true
      } catch (err) {
        log('error: failed to start the indexer', err)
      }
    }
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector...')
    await this.indexer.stop()
  }

  private async sendTransactionInternal(txPayload: TransactionPayload, eventName: IndexerEventsNames) {
    return await this.chain.sendTransaction(true, txPayload, (txHash: string) =>
      this.setTxHandler(`${eventName}${txHash}`, txHash)
    )
  }

  public async sendTransaction(txPayload: WasmTransactionPayload, eventName: IndexerEventsNames) {
    let innerPayload: TransactionPayload = {
      data: txPayload.data,
      to: txPayload.to,
      value: txPayload.value != '' ? BigNumber.from(txPayload.value) : BigNumber.from(0)
    }

    let txResult = await this.sendTransactionInternal(innerPayload, eventName)

    if (txResult.code == SendTransactionStatus.SUCCESS) {
      return {
        code: 'SUCCESS',
        tx: txResult.tx.hash
      }
    } else {
      return {
        code: txResult.code.toString(),
        tx: undefined
      }
    }
  }

  public setTxHandler(evt: IndexerEventsType, tx: string): DeferType<string> {
    return this.indexer.resolvePendingTransaction(evt, tx)
  }

  public getOpenChannelsFrom(p: Address) {
    return this.indexer.getOpenChannelsFrom(p)
  }

  public async getAccount(addr: Address) {
    return this.indexer.getAccount(addr)
  }

  public getChainKeyOf(addr: Address) {
    return this.indexer.getChainKeyOf(addr)
  }

  public getPacketKeyOf(addr: Address) {
    return this.indexer.getPacketKeyOf(addr)
  }

  public getRandomOpenChannel() {
    return this.indexer.getRandomOpenChannel()
  }

  /**
   * Retrieves HOPR balance of the node, optionally uses the indexer.
   * The difference from the two methods is that the latter relys on
   * the coming events which require 8 blocks to be confirmed.
   * @returns HOPR balance
   */
  public async getBalance(useIndexer: boolean = false): Promise<Balance> {
    let ret
    if (useIndexer) {
      ret = await this.db.get_hopr_balance()
    } else {
      let selfAddr = this.chainKeypair.to_address()
      ret = this.chain.getBalance(selfAddr)
    }
    return ret
  }

  public getPublicKey(): PublicKey {
    return this.chainKeypair.public()
  }

  /**
   * Retrieves HOPR balance of the safe.
   * @returns HOPR balance
   */
  public async getSafeBalance(): Promise<Balance> {
    return this.chain.getBalance(this.safeModuleOptions.safeAddress)
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = (address: string) => {
    return this.chain.getNativeBalance(Address.from_string(address))
  }

  private cachedGetNativeBalance = (address: string) =>
    cacheNoArgAsyncFunction<Balance>(() => this.uncachedGetNativeBalance(address), constants.PROVIDER_CACHE_TTL)

  public async getNativeBalance(address: string, useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetNativeBalance(address)() : this.uncachedGetNativeBalance(address)
  }

  public smartContractInfo(): {
    chain: string
    hoprAnnouncementsAddress: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    hoprNetworkRegistryAddress: string
    hoprNodeSafeRegistryAddress: string
    hoprTicketPriceOracleAddress: string
    moduleAddress: string
    safeAddress: string
    noticePeriodChannelClosure: number
  } {
    return this.chain.getInfo()
  }

  public async isSafeAnnouncementAllowed(): Promise<boolean> {
    // the position comes from the order of values in the smart contract
    const ALLOW_ALL_ENUM_POSITION = 3
    const targetAddress = this.chain.getInfo().hoprAnnouncementsAddress
    let target
    try {
      target = await this.chain.getNodeManagementModuleTargetInfo(targetAddress)
    } catch (err) {
      log(`Error getting module target info for address ${targetAddress}: ${err}`)
    }
    if (target) {
      const targetAddress2 = target.shr(96)
      const targetPermission = target.shl(176).shr(248)
      const addressMatches = BigNumber.from(targetAddress).eq(targetAddress2)
      const permissionIsAllowAll = BigInt.asUintN(8, targetPermission.toBigInt()) == BigInt(ALLOW_ALL_ENUM_POSITION)
      return addressMatches && permissionIsAllowAll
    }
    return false
  }

  public async isNodeSafeRegisteredCorrectly(): Promise<boolean> {
    const nodeAddress = this.chainKeypair.to_address()
    const safeAddress = this.safeModuleOptions.safeAddress
    const registeredAddress = await this.chain.getSafeFromNodeSafeRegistry(nodeAddress)
    log('Currently registered Safe address in NodeSafeRegistry = %s', registeredAddress.to_string())
    return registeredAddress.eq(Address.deserialize(safeAddress.serialize()))
  }

  public async isNodeSafeNotRegistered(): Promise<boolean> {
    const nodeAddress = this.chainKeypair.to_address()
    const registeredAddress = await this.chain.getSafeFromNodeSafeRegistry(nodeAddress)
    log('Currently registered Safe address in NodeSafeRegistry = %s', registeredAddress.to_string())
    return registeredAddress.eq(new Address(new Uint8Array(Address.size()).fill(0x00)))
  }

  public async canRegisterWithSafe() {
    const nodeAddress = this.chainKeypair.to_address()
    const safeAddress = this.safeModuleOptions.safeAddress
    log(`====> registerSafeByNode nodeAddress: ${nodeAddress.to_hex()} safeAddress ${safeAddress.to_hex()}`)

    const targetAddress = await this.chain.getModuleTargetAddress()
    if (!targetAddress.eq(Address.from_string(safeAddress.to_string()))) {
      // cannot proceed when the safe address is not the target/owner of given module
      throw Error('Safe is not a target of module.')
    }

    const registeredAddress = await this.chain.getSafeFromNodeSafeRegistry(nodeAddress)
    log('Currently registered Safe address in NodeSafeRegistry = %s', registeredAddress.to_string())

    if (registeredAddress.eq(new Address(new Uint8Array(Address.size()).fill(0x00)))) {
      log('Node is not associated with a Safe in NodeSafeRegistry yet')
      return true
    } else if (!registeredAddress.eq(safeAddress)) {
      throw Error('Node is associated with a different Safe in NodeSafeRegistry')
    } else {
      log('Node is associated with correct Safe in NodeSafeRegistry')
    }

    return false
  }

  public static createMockInstance(chainKeypair: ChainKeypair, peerId: PeerId): HoprCoreEthereum {
    const connectorLogger = debug(`hopr:mocks:connector`)
    //const packetSecret = "1d6689707dfff6a93b206b3f5addcaa8789a1812e43fb393f8ad02f54ddf599d"
    HoprCoreEthereum._instance = {
      start: () => {
        connectorLogger('starting connector called.')
        return {} as unknown as HoprCoreEthereum
      },
      stop: () => {
        connectorLogger('stopping connector called.')
        return Promise.resolve()
      },
      getNativeBalance: () => {
        connectorLogger('getNativeBalance method was called')
        return Promise.resolve(new Balance('10000000000000000000', BalanceType.Native))
      },
      getPublicKey: () => {
        connectorLogger('getPublicKey method was called')
        return chainKeypair.public()
      },
      getAccount: () => {
        connectorLogger('getAccount method was called')
        return Promise.resolve(
          new AccountEntry(
            OffchainPublicKey.from_peerid_str(peerId.toString()),
            chainKeypair.public().to_address(),
            `/ip4/127.0.0.1/tcp/124/p2p/${peerId.toString()}`,
            1
          )
        )
      },
      waitForPublicNodes: () => {
        connectorLogger('On-chain request for existing public nodes.')
        return Promise.resolve([])
      },
      on: (event: string) => {
        connectorLogger(`On-chain signal for event "${event}"`)
      },
      indexer: {
        on: (event: string) => connectorLogger(`Indexer on handler top of chain called with event "${event}"`),
        off: (event: string) => connectorLogger(`Indexer off handler top of chain called with event "${event}`)
      },
      isAllowedAccessToNetwork: () => Promise.resolve(true)
    } as unknown as HoprCoreEthereum

    return HoprCoreEthereum._instance
  }
}

// export { useFixtures } from './indexer/index.mock.js'
export { sampleChainOptions } from './ethereum.mock.js'

export { ChannelEntry, Indexer, ChainWrapper, createChainWrapper, SmartContractConfig, Hash }
