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
  ChannelStatus,
  generate_channel_id,
  Hash,
  debug,
  ChannelEntry,
  type DeferType,
  PublicKey,
  AccountEntry,
  create_counter,
  OffchainPublicKey,
  ChainKeypair,
  OffchainKeypair,
  is_allowed_to_access_network,
  CORE_ETHEREUM_CONSTANTS,
  Database
} from '@hoprnet/hopr-utils'

import Indexer from './indexer/index.js'
import { EventEmitter } from 'events'
import type { IndexerEvents } from './indexer/types.js'

const log = debug('hopr-core-ethereum')

export type RedeemTicketResponse =
  | {
      status: 'SUCCESS'
      receipt: string
      ackTicket: AcknowledgedTicket
    }
  | {
      status: 'FAILURE'
      message: string
    }
  | {
      status: 'ERROR'
      error: Error | string
    }

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

type ticketRedemtionInChannelOperations = Map<string, Promise<void>>

// Exported from Rust
const constants = CORE_ETHEREUM_CONSTANTS()

// Metrics
const metric_losingTickets = create_counter('core_ethereum_counter_losing_tickets', 'Number of losing tickets')
const metric_winningTickets = create_counter('core_ethereum_counter_winning_tickets', 'Number of winning tickets')

export default class HoprCoreEthereum extends EventEmitter {
  private static _instance: HoprCoreEthereum

  public indexer: Indexer
  private chain: ChainWrapper
  private started: Promise<HoprCoreEthereum> | undefined
  private redeemingAll: Promise<void> | undefined = undefined
  // Used to store ongoing operations to prevent duplicate redemption attempts
  private ticketRedemtionInChannelOperations: ticketRedemtionInChannelOperations = new Map()

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

        // update token balance
        //const hoprBalance = await this.chain.getBalance(this.chainKeypair.to_address())
        //await this.db.set_hopr_balance(hoprBalance)
        //log(`set own HOPR balance to ${hoprBalance.to_formatted_string()}`)

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

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    return this.chain.withdraw(currency, recipient, amount, (tx: string) =>
      this.setTxHandler(currency === 'NATIVE' ? `withdraw-native-${tx}` : `withdraw-hopr-${tx}`, tx)
    )
  }

  public setTxHandler(evt: IndexerEvents, tx: string): DeferType<string> {
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

  public async waitForPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return await this.indexer.getPublicNodes()
  }

  public async redeemAllTickets(): Promise<void> {
    if (this.redeemingAll) {
      log('skipping redeemAllTickets because another operation is still in progress')
      return this.redeemingAll
    }

    return new Promise((resolve, reject) => {
      try {
        this.redeemingAll = this.redeemAllTicketsInternalLoop().then(resolve, reject)
      } catch (err) {
        reject(err)
      }
    })
  }

  private async redeemAllTicketsInternalLoop(): Promise<void> {
    try {
      let channelsTo = await this.db.get_channels_to(this.chainKeypair.to_address())
      while (channelsTo.len() > 0) {
        let channel = channelsTo.next()
        await this.redeemTicketsInChannel(ChannelEntry.deserialize(channel.serialize()))
      }
    } catch (err) {
      log(`error during redeeming all tickets`, err)
    }

    // whenever we finish this loop we clear the reference
    this.redeemingAll = undefined
  }

  public async redeemTicketsInChannelByCounterparty(counterparty: Address) {
    const channel = await this.db.get_channel_from(counterparty)
    return this.redeemTicketsInChannel(channel)
  }

  public async redeemTicketsInChannel(channel: ChannelEntry) {
    const channelId = channel.get_id().to_hex()
    const currentOperation = this.ticketRedemtionInChannelOperations.get(channelId)

    // verify that no operation is running, or return the active operation
    if (currentOperation) {
      log(`redemption of tickets in channel ${channelId} is currently in progress`)
      return currentOperation
    }

    log(`starting new ticket redemption in channel ${channelId}`)
    // start new operation and store it
    return new Promise((resolve, reject) => {
      try {
        this.ticketRedemtionInChannelOperations.set(
          channelId,
          this.redeemTicketsInChannelLoop(channel).then(resolve, reject)
        )
      } catch (err) {
        reject(err)
      }
    })
  }

  private async redeemTicketsInChannelLoop(channel: ChannelEntry): Promise<void> {
    const channelId = channel.get_id()
    if (!channel.destination.eq(this.chainKeypair.to_address())) {
      // delete operation before returning
      this.ticketRedemtionInChannelOperations.delete(channelId.to_hex())
      throw new Error('Cannot redeem ticket in channel that is not to us')
    }

    log(`Going to redeem tickets in channel ${channelId.to_hex()}`)

    // Because tickets are ordered and require the previous redemption to
    // have succeeded before we can redeem the next, we need to do this
    // sequentially.
    // We redeem step-wise, reading only the next ticket from the db, to
    // reduce the chance for race-conditions with db write operations on
    // those tickets.

    const boundRedeemTicket = this.redeemTicket.bind(this)
    const boundGetAckdTickets = this.db.get_acknowledged_tickets.bind(this.db)
    const boundMarkLosingAckedTicket = this.db.mark_losing_acked_ticket.bind(this.db)

    // Use an async iterator to make execution interruptable and allow
    // Node.JS to schedule iterations at any time
    const ticketRedeemIterator = async function* () {
      let serdeChannel = channel
      let tickets = await boundGetAckdTickets(serdeChannel)
      log(`there are ${tickets.len()} left to redeem in channel ${channelId.to_hex()}`)

      let ticket: AcknowledgedTicket
      while (tickets.len() > 0) {
        let fetched = tickets.next()
        log(`fetched ticket with index ${fetched.ticket.index.to_string()}`)
        if (ticket != undefined && ticket.ticket.index.eq(fetched.ticket.index)) {
          // @TODO handle errors
          log(
            `Fetched ticket with the same index ${ticket.ticket.index.to_string()} in channel ${channelId.to_hex()}. Giving up.`
          )
          break
        }

        ticket = fetched

        log(
          `redeeming ticket ${ticket.response.to_hex()} in channel ${channelId.to_hex()} from ${channel.source.to_hex()} to ${channel.destination.to_hex()}, porSecret ${ticket.response.to_hex()}`
        )

        log(ticket.ticket.to_string())

        const result = await boundRedeemTicket(channel.source, channelId, ticket)

        if (result.status !== 'SUCCESS') {
          if (result.status === 'ERROR') {
            // We need to abort as tickets require ordered redemption.
            // delete operation before returning
            log(
              `error while redeeming ticket ${ticket.ticket.index.to_string()} in channel ${channelId.to_hex()}: ${result.error.toString()}`
            )
            throw result.error
          } else {
            // result.status === 'FAILURE'
            // May fail due to out-of-commits, preimage-is-empty, not-a-winning-ticket
            // Treat those acked tickets as losing tickets, and remove them from the DB.
            log(
              `redemption of ticket ${
                ticket.ticket.index
              } failed in channel ${channelId.to_hex()} - marking it as losing: ${result.message}`
            )
            await boundMarkLosingAckedTicket(ticket)
            metric_losingTickets.increment()
          }
        }

        yield ticket.response

        tickets = await boundGetAckdTickets(serdeChannel)
        log(`yet there are ${tickets.len()} left to redeem in channel ${channelId.to_hex()}`)
      }
    }

    try {
      for await (const ticketResponse of ticketRedeemIterator()) {
        log(`ticket ${ticketResponse.to_hex()} in channel ${channelId.to_hex()} was redeemed`)
      }
      log(`redemption of tickets from ${channel.source.to_string()} in channel ${channelId.to_hex()} is complete`)
    } catch (err) {
      log(`redemption of tickets from ${channel.source.to_string()} in channel ${channelId.to_hex()} failed`, err)
    } finally {
      this.ticketRedemtionInChannelOperations.delete(channelId.to_hex())
    }
  }

  public async redeemTicket(
    counterparty: Address,
    channelId: Hash,
    ackTicket: AcknowledgedTicket
  ): Promise<RedeemTicketResponse> {
    let receipt: string
    try {
      log(
        `Performing ticket redemption ticket for counterparty ${counterparty.to_hex()} in channel ${channelId.to_hex()}`
      )
      await this.chain.redeemTicket(counterparty, ackTicket, (txHash: string) =>
        this.setTxHandler(`channel-updated-${txHash}`, txHash)
      )
      await this.db.mark_redeemed(counterparty, ackTicket)

      log(`redeemed ticket for counterparty ${counterparty.to_hex()}`)
    } catch (err) {
      log(`ticket redemption error: ${err.toString()}`)
      return {
        status: 'ERROR',
        error: err.toString()
      }
    }

    metric_winningTickets.increment()

    this.emit('ticket:redeemed', ackTicket)

    return {
      status: 'SUCCESS',
      ackTicket,
      receipt
    }
  }

  async initializeClosure(src: Address, dest: Address): Promise<string> {
    // TODO: should remove this blocker when https://github.com/hoprnet/hoprnet/issues/4194 gets addressed
    if (!this.chainKeypair.to_address().eq(src)) {
      throw Error('Initialize incoming channel closure currently is not supported.')
    }

    const c = ChannelEntry.deserialize((await this.db.get_channel_x(src, dest)).serialize())

    if (c.status !== ChannelStatus.Open) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    return this.chain.initiateOutgoingChannelClosure(dest, (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  public async finalizeClosure(src: Address, dest: Address): Promise<string> {
    // TODO: should remove this blocker when https://github.com/hoprnet/hoprnet/issues/4194 gets addressed
    if (!this.chainKeypair.to_address().eq(src)) {
      throw Error('Finalizing incoming channel closure currently is not supported.')
    }
    const c = ChannelEntry.deserialize((await this.db.get_channel_x(src, dest)).serialize())

    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeOutgoingChannelClosure(dest, (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  public async openChannel(dest: Address, amount: Balance): Promise<{ channelId: Hash; receipt: Receipt }> {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.db.get_channel_to(dest)
    } catch {
      log(`failed to retrieve channel information`)
    }

    if (c && c.status !== ChannelStatus.Closed) {
      throw Error('Channel is already opened')
    }

    const myBalance = await this.getSafeBalance()
    if (myBalance.lt(amount)) {
      throw Error('We do not have enough balance to open a channel')
    }

    log(`opening channel to ${dest.to_hex()} with amount ${amount.to_formatted_string()}`)

    const allowance = Balance.deserialize(
      (await this.db.get_staking_safe_allowance()).serialize_value(),
      BalanceType.HOPR
    )
    if (allowance.lt(myBalance)) {
      throw Error('We do not have enough allowance to fund the channel')
    }

    const receipt = await this.fundChannel(dest, amount, Balance.zero(BalanceType.HOPR))
    return { channelId: generate_channel_id(this.chainKeypair.to_address(), dest), receipt }
  }

  public async fundChannel(dest: Address, myFund: Balance, counterpartyFund: Balance): Promise<Receipt> {
    const totalFund = myFund.add(counterpartyFund)
    const myBalance = await this.getSafeBalance()
    if (totalFund.gt(myBalance)) {
      throw Error('We do not have enough balance to fund the channel')
    }
    log(
      `====> fundChannel: src: ${this.chainKeypair
        .to_address()
        .to_string()} dest: ${dest.to_string()} amount: ${myFund.to_string()} | ${counterpartyFund.to_string()}`
    )

    const allowance = Balance.deserialize(
      (await this.db.get_staking_safe_allowance()).serialize_value(),
      BalanceType.HOPR
    )
    if (allowance.lt(myFund)) {
      throw Error('We do not have enough allowance to fund the channel')
    }
    return (
      await this.chain.fundChannel(
        dest,
        myFund,
        (txHash: string) => this.setTxHandler(`channel-updated-${txHash}`, txHash)
        // we are only interested in fundChannel receipt
      )
    )[1]
  }

  public async isSafeAnnouncementAllowed(): Promise<boolean> {
    // the position comes from the order of values in the smart contract
    const ALLOW_ALL_ENUM_POSITION = 3
    const targetAddress = this.chain.getInfo().hoprAnnouncementsAddress
    const target = await this.chain.getNodeManagementModuleTargetInfo(targetAddress)
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

  /**
   * Checks whether a given `hoprNode` is allowed access.
   * When the register is disabled, a `hoprNode` is seen as `registered`,
   * when the register is enabled, a `hoprNode` needs to also be `eligible`.
   * @param hoprNode Ethereum address of the account we want to check if it's registered
   * @returns true if registered
   */
  public async isAllowedAccessToNetwork(hoprNode: Address): Promise<boolean> {
    return await is_allowed_to_access_network(this.db, hoprNode)
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
        off: (event: string) => connectorLogger(`Indexer off handler top of chain called with event "${event}`),
        getPublicNodes: () => Promise.resolve([])
      },
      isAllowedAccessToNetwork: () => Promise.resolve(true)
    } as unknown as HoprCoreEthereum

    return HoprCoreEthereum._instance
  }
}

// export { useFixtures } from './indexer/index.mock.js'
export { sampleChainOptions } from './ethereum.mock.js'

export { ChannelEntry, Indexer, ChainWrapper, createChainWrapper, DeploymentExtract, Hash }
