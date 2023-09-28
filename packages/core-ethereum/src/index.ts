import { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'
import { ChainWrapper, createChainWrapper, Receipt, type DeploymentExtract } from './ethereum.js'
import { BigNumber } from 'ethers'
import {
  AcknowledgedTicket,
  Balance,
  BalanceType,
  Address,
  cacheNoArgAsyncFunction,
  generate_channel_id,
  Hash,
  debug,
  ChannelEntry,
  type DeferType,
  PublicKey,
  AccountEntry,
  OffchainPublicKey,
  ChainKeypair,
  OffchainKeypair,
  CORE_ETHEREUM_CONSTANTS,
  Database
} from '@hoprnet/hopr-utils'

import Indexer from './indexer/index.js'
import { EventEmitter } from 'events'
import type { IndexerEventsType } from './indexer/types.js'
export {
  BlockEventName,
  BlockProcessedEventName,
  StatusEventName,
  PeerEventName,
  NetworkRegistryEligibilityChangedEventName,
  NetworkRegistryStatusChangedEventName,
  NetworkRegistryNodeAllowedEventName,
  NetworkRegistryNodeNotAllowedEventName
} from './indexer/types.js'

const log = debug('hopr-core-ethereum')

export type ChainOptions = {
  provider: string
  maxConfirmations?: number
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
  private started: Promise<HoprCoreEthereum> | undefined
  // Used to store ongoing operations to prevent duplicate redemption attempts

  private constructor(
    private db: Database,
    private offchainKeypair: OffchainKeypair,
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
      this.options.maxConfirmations ?? constants.DEFAULT_CONFIRMATIONS,
      constants.INDEXER_BLOCK_RANGE
    )
  }

  public static async createInstance(
    db: Database,
    offchainKeypair: OffchainKeypair,
    chainKeypair: ChainKeypair,
    options: ChainOptions,
    safeModuleOptions: SafeModuleOptions,
    deploymentAddresses: DeploymentExtract,
    automaticChainCreation = true
  ) {
    HoprCoreEthereum._instance = new HoprCoreEthereum(
      db,
      offchainKeypair,
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

  async initializeChainWrapper(deploymentAddresses: DeploymentExtract) {
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

  private async createChain(deploymentAddresses: DeploymentExtract): Promise<void> {
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
        this.offchainKeypair,
        this.chainKeypair,
        true
      )
    } catch (err) {
      const errMsg = 'failed to create provider chain wrapper'
      log(`error: ${errMsg}`, err)
      throw Error(errMsg)
    }

    // Emit event to make sure connector is aware the chain was created properly.
    this.emit('hopr:connector:created')
  }

  async start(): Promise<HoprCoreEthereum> {
    if (this.started) {
      return this.started
    }

    const _start = async (): Promise<HoprCoreEthereum> => {
      try {
        await this.chain.waitUntilReady()

        // indexer starts
        await this.indexer.start(this.chain, this.chain.getGenesisBlock(), this.safeModuleOptions.safeAddress)

        // Debug log used in e2e integration tests, please don't change
        log(`using blockchain address ${this.chainKeypair.to_address().to_hex()}`)
        log('Connector started')
      } catch (err) {
        log('error: failed to start the indexer', err)
      }
      return this
    }
    this.started = _start()
    return this.started
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector...')
    await this.indexer.stop()
  }

  announce(multiaddr: Multiaddr, useSafe: boolean = false): Promise<string> {
    // Currently we announce always with key bindings
    return this.chain.announce(multiaddr, useSafe, (txHash: string) => this.setTxHandler(`announce-${txHash}`, txHash))
  }

  async withdraw(recipient: Address, amount: Balance): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    let currency: 'NATIVE' | 'HOPR' = amount.balance_type() == BalanceType.Native ? 'NATIVE' : 'HOPR'

    return this.chain.withdraw(currency, recipient.to_string(), amount.amount().to_string(), (tx: string) =>
      this.setTxHandler(currency === 'NATIVE' ? `withdraw-native-${tx}` : `withdraw-hopr-${tx}`, tx)
    )
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
    return useIndexer
      ? Balance.deserialize((await this.db.get_hopr_balance()).serialize_value(), BalanceType.HOPR)
      : this.chain.getBalance(this.chainKeypair.to_address())
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

  public async sendTicketRedeemTx(ackTicket: AcknowledgedTicket): Promise<string> {
    try {
      return await this.chain.redeemTicket(ackTicket, (txHash: string) => {
        return this.setTxHandler(`channel-updated-${txHash}`, txHash)
      })
    } catch (err) {
      log(`ticket redemption error: ${err.toString()}`)
      throw new Error(`ticket redemption error: ${err.toString()}`)
    }
  }

  async initializeClosure(_src: Address, dest: Address): Promise<string> {
    return this.chain.initiateOutgoingChannelClosure(dest, (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  async finalizeClosure(_src: Address, dest: Address): Promise<string> {
    return await this.chain.finalizeOutgoingChannelClosure(dest, (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  async openChannel(dest: Address, amount: Balance): Promise<{ channel_id: string; receipt: string }> {
    log(`opening channel to ${dest.to_hex()} with amount ${amount.to_formatted_string()}`)
    const receipt = await this.fundChannel(dest, amount)
    return { channel_id: generate_channel_id(this.chainKeypair.to_address(), dest).to_hex(), receipt }
  }

  // This operation works on open and closed channels. More assertions must be
  // enforced on a higher layer.
  async fundChannel(dest: Address, amount: Balance): Promise<Receipt> {
    const myBalance = await this.getSafeBalance()

    if (amount.gt(myBalance)) {
      throw Error(`Not enough balance (${myBalance.to_string()} < ${amount.to_string()})`)
    }
    log(
      `====> fundChannel: src: ${this.chainKeypair
        .to_address()
        .to_string()} dest: ${dest.to_string()} amount: ${amount.to_string()}`
    )

    const receipt = (
      await this.chain.fundChannel(
        dest,
        amount,
        (txHash: string) => this.setTxHandler(`channel-updated-${txHash}`, txHash)
        // we are only interested in fundChannel receipt
      )
    )[1]
    return receipt
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

  public async registerSafeByNode(): Promise<Receipt> {
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

    let receipt = undefined
    if (registeredAddress.eq(new Address(new Uint8Array(Address.size()).fill(0x00)))) {
      log('Node is not associated with a Safe in NodeSafeRegistry yet')
      receipt = await this.chain.registerSafeByNode(safeAddress, (txHash: string) =>
        this.setTxHandler(`node-safe-registered-${txHash}`, txHash)
      )
    } else if (!registeredAddress.eq(Address.deserialize(safeAddress.serialize()))) {
      // the node has been associated with a differnt safe address
      log('Node is associated with a different Safe in NodeSafeRegistry')
      throw Error('Node has been registered with a different safe')
    } else {
      log('Node is associated with correct Safe in NodeSafeRegistry')
    }

    log('update safe and module addresses in database')
    await this.db.set_staking_safe_address(safeAddress)
    await this.db.set_staking_module_address(this.safeModuleOptions.moduleAddress)

    return receipt
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
      announce: () => {
        connectorLogger('On-chain announce request sent')
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

export { ChannelEntry, Indexer, ChainWrapper, createChainWrapper, DeploymentExtract, Hash }
