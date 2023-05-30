import { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'
import { ChainWrapper, createChainWrapper, Receipt } from './ethereum.js'
import chalk from 'chalk'
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
  privKeyToPeerId,
  type ChannelEntry,
  type DeferType,
  PublicKey,
  AccountEntry,
  create_counter
} from '@hoprnet/hopr-utils'
import { Database as Ethereum_Database } from '../lib/core_ethereum_db.js'
import Indexer from './indexer/index.js'
import { CORE_ETHEREUM_CONSTANTS } from '../lib/core_ethereum_misc.js'
import { EventEmitter } from 'events'
import { initializeCommitment, findCommitmentPreImage, bumpCommitment, ChannelCommitmentInfo } from './commitment.js'
import type { IndexerEvents } from './indexer/types.js'
import { DeploymentExtract } from './utils/utils.js'

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
    private db: Ethereum_Database,
    private publicKey: PublicKey,
    private privateKey: Uint8Array,
    private options: ChainOptions,
    private automaticChainCreation: boolean
  ) {
    super()

    log(`[DEBUG] initialized Rust DB... ${JSON.stringify(this.db.toString(), null, 2)} `)

    this.indexer = new Indexer(
      this.publicKey.to_address(),
      this.db,
      this.options.maxConfirmations ?? constants.DEFAULT_CONFIRMATIONS,
      constants.INDEXER_BLOCK_RANGE
    )
  }

  public static createInstance(
    db: Ethereum_Database,
    publicKey: PublicKey,
    privateKey: Uint8Array,
    options: ChainOptions,
    automaticChainCreation = true
  ) {
    HoprCoreEthereum._instance = new HoprCoreEthereum(
      db,
      publicKey,
      privateKey,
      options,
      automaticChainCreation
    )
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
      this.chain = await createChainWrapper(deploymentAddresses, this.options, this.privateKey, true)
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

        const hoprBalance = await this.chain.getBalance(this.publicKey.to_address())
        await this.db.set_hopr_balance(hoprBalance)
        log(`set own HOPR balance to ${hoprBalance.to_formatted_string()}`)

        await this.indexer.start(this.chain, this.chain.getGenesisBlock())

        // Debug log used in e2e integration tests, please don't change
        log(`using blockchain address ${this.publicKey.to_address().to_hex()}`)
        log(chalk.green('Connector started'))
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

  announce(multiaddr: Multiaddr): Promise<string> {
    return this.chain.announce(multiaddr, (txHash: string) => this.setTxHandler(`announce-${txHash}`, txHash))
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

  public getOpenChannelsFrom(p: PublicKey) {
    return this.indexer.getOpenChannelsFrom(p)
  }

  public async getAccount(addr: Address) {
    return this.indexer.getAccount(addr)
  }

  public getPublicKeyOf(addr: Address) {
    return this.indexer.getPublicKeyOf(addr)
  }

  public getRandomOpenChannel() {
    return this.indexer.getRandomOpenChannel()
  }

  /**
   * Retrieves HOPR balance, optionally uses the indexer.
   * The difference from the two methods is that the latter relys on
   * the coming events which require 8 blocks to be confirmed.
   * @returns HOPR balance
   */
  public async getBalance(useIndexer: boolean = false): Promise<Balance> {
    return useIndexer ? this.db.get_hopr_balance() : this.chain.getBalance(this.publicKey.to_address())
  }

  public getPublicKey(): PublicKey {
    return this.publicKey
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => {
    return this.chain.getNativeBalance(this.publicKey.to_address())
  }
  private cachedGetNativeBalance = cacheNoArgAsyncFunction<Balance>(
    this.uncachedGetNativeBalance,
    constants.PROVIDER_CACHE_TTL
  )
  public async getNativeBalance(useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetNativeBalance() : this.uncachedGetNativeBalance()
  }

  public smartContractInfo(): {
    chain: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    hoprNetworkRegistryAddress: string
    channelClosureSecs: number
  } {
    return this.chain.getInfo()
  }

  public async waitForPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return await this.indexer.getPublicNodes()
  }

  public async commitToChannel(c: ChannelEntry): Promise<void> {
    log(`committing to channel ${c.get_id().to_hex()}`)
    log(c.toString())
    const setCommitment = async (commitment: Hash) => {
      return this.chain.setCommitment(c.source.to_address(), commitment, (txHash: string) =>
        this.setTxHandler(`channel-updated-${txHash}`, txHash)
      )
    }
    const getCommitment = async () => (await this.db.get_channel(c.get_id())).commitment

    // Get all channel information required to build the initial commitment
    const cci = new ChannelCommitmentInfo(
      this.options.chainId,
      this.smartContractInfo().hoprChannelsAddress,
      c.get_id(),
      c.channel_epoch
    )

    await initializeCommitment(this.db, privKeyToPeerId(this.privateKey), cci, getCommitment, setCommitment)
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
      let channelsTo = await this.db.get_channels_to(this.publicKey.to_address())
      while (channelsTo.len() > 0) {
        let channel = channelsTo.next()
        await this.redeemTicketsInChannel(channel)
      }
    } catch (err) {
      log(`error during redeeming all tickets`, err)
    }

    // whenever we finish this loop we clear the reference
    this.redeemingAll = undefined
  }

  public async redeemTicketsInChannelByCounterparty(counterparty: PublicKey) {
    const channel = await this.db.get_channel_from(counterparty)
    return this.redeemTicketsInChannel(channel)
  }

  public async redeemTicketsInChannel(channel: ChannelEntry) {
    const channelId = channel.get_id().to_hex()
    const currentOperation = this.ticketRedemtionInChannelOperations.get(channelId)

    // verify that no operation is running, or return the active operation
    if (currentOperation) {
      return currentOperation
    }

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
    if (!channel.destination.eq(this.getPublicKey())) {
      // delete operation before returning
      this.ticketRedemtionInChannelOperations.delete(channelId.to_hex())
      throw new Error('Cannot redeem ticket in channel that is not to us')
    }

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
      let tickets = await boundGetAckdTickets({ channel })
      let ticket: AcknowledgedTicket
      while (tickets.len() > 0) {
        let fetched = tickets.next()
        if (ticket != undefined && ticket.ticket.index.eq(fetched.ticket.index)) {
          // @TODO handle errors
          log(
            `Could not redeem ticket with index ${ticket.ticket.index.to_string()} in channel ${channelId.to_hex()}. Giving up.`
          )
          break
        }

        ticket = fetched

        log(
          `redeeming ticket ${ticket.response.to_hex()} in channel from ${channel.source} to ${
            channel.destination
          }, preImage ${ticket.pre_image.to_hex()}, porSecret ${ticket.response.to_hex()}`
        )

        log(ticket.ticket.toString())

        const result = await boundRedeemTicket(channel.source, channelId, ticket)

        if (result.status !== 'SUCCESS') {
          if (result.status === 'ERROR') {
            // We need to abort as tickets require ordered redemption.
            // delete operation before returning
            throw result.error
          } else {
            // May fail due to out-of-commits, preimage-is-empty, not-a-winning-ticket
            // Treat those acked tickets as losing tickets, and remove them from the DB.
            await boundMarkLosingAckedTicket(ticket)
            metric_losingTickets.increment()
          }
        }

        yield ticket.response

        tickets = await boundGetAckdTickets({ channel })
      }
    }

    try {
      for await (const ticketResponse of ticketRedeemIterator()) {
        log(`ticket ${ticketResponse.to_hex()} was redeemed`)
      }
      log(`redemption of tickets from ${channel.source.toString()} is complete`)
    } catch (err) {
      log(`redemption of tickets from ${channel.source.toString()} failed`, err)
    } finally {
      this.ticketRedemtionInChannelOperations.delete(channelId.to_hex())
    }
  }

  public async redeemTicket(
    counterparty: PublicKey,
    channelId: Hash,
    ackTicket: AcknowledgedTicket
  ): Promise<RedeemTicketResponse> {
    if (!ackTicket.verify(counterparty)) {
      return {
        status: 'FAILURE',
        message: 'Invalid response to acknowledgement'
      }
    }

    let commitmentPreImage: Hash // actual ackTicket.preImage

    try {
      commitmentPreImage = await findCommitmentPreImage(this.db, channelId)
    } catch (err) {
      log(`Channel ${channelId.to_hex()} is out of commitments`)
      // TODO: How should we handle this ticket if it's out of commitment
      return {
        status: 'ERROR',
        error: err
      }
    }
    // set the commitment
    ackTicket.set_preimage(commitmentPreImage)
    log(`Set preImage ${commitmentPreImage.to_hex()} for ticket ${ackTicket.response.to_hex()}`)

    let receipt: string

    try {
      const ticket = ackTicket.ticket
      log('Submitting ticket', ackTicket.response.to_hex())
      const emptyPreImage = new Hash(new Uint8Array(Hash.size()).fill(0x00))
      const hasPreImage = !ackTicket.pre_image.eq(emptyPreImage)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ackTicket.response.to_hex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message: 'PreImage is empty.'
        }
      }

      const isWinning = ticket.is_winning(ackTicket.pre_image, ackTicket.response, ticket.win_prob)

      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.to_hex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      // address winning ticket
      metric_winningTickets.increment()

      receipt = await this.chain.redeemTicket(counterparty.to_address(), ackTicket, (txHash: string) =>
        this.setTxHandler(`channel-updated-${txHash}`, txHash)
      )
    } catch (err) {
      // TODO delete ackTicket -- check if it's due to gas!
      log('Unexpected error when redeeming ticket', ackTicket.response.to_hex(), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
    log('Successfully submitted ticket', ackTicket.response.to_hex())

    // bump commitment when on-chain ticket redemption is successful
    // FIXME: bump commitment can fail if channel runs out of commitments
    await bumpCommitment(this.db, channelId, commitmentPreImage)
    log(`Successfully bump local commitment after ${commitmentPreImage.to_hex()}`)

    await this.db.mark_redeemed(ackTicket)
    this.emit('ticket:redeemed', ackTicket)
    return {
      status: 'SUCCESS',
      receipt,
      ackTicket
    }
  }

  async initializeClosure(src: PublicKey, dest: PublicKey): Promise<string> {
    // TODO: should remove this blocker when https://github.com/hoprnet/hoprnet/issues/4194 gets addressed
    if (!this.publicKey.eq(src)) {
      throw Error('Initialize incoming channel closure currently is not supported.')
    }

    const c = await this.db.get_channel_x(src, dest)
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    return this.chain.initiateChannelClosure(dest.to_address(), (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  public async finalizeClosure(src: PublicKey, dest: PublicKey): Promise<string> {
    // TODO: should remove this blocker when https://github.com/hoprnet/hoprnet/issues/4194 gets addressed
    if (!this.publicKey.eq(src)) {
      throw Error('Finalizing incoming channel closure currently is not supported.')
    }
    const c = await this.db.get_channel_x(src, dest)
    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(dest.to_address(), (txHash: string) =>
      this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  public async openChannel(dest: PublicKey, amount: Balance): Promise<{ channelId: Hash; receipt: Receipt }> {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.db.get_channel_to(dest)
    } catch {}
    if (c && c.status !== ChannelStatus.Closed) {
      throw Error('Channel is already opened')
    }

    const myBalance = await this.getBalance()
    if (myBalance.lt(amount)) {
      throw Error('We do not have enough balance to open a channel')
    }
    const receipt = await this.fundChannel(dest, amount, Balance.zero(BalanceType.HOPR))
    return { channelId: generate_channel_id(this.publicKey.to_address(), dest.to_address()), receipt }
  }

  public async fundChannel(dest: PublicKey, myFund: Balance, counterpartyFund: Balance): Promise<Receipt> {
    const totalFund = myFund.add(counterpartyFund)
    const myBalance = await this.getBalance()
    if (totalFund.gt(myBalance)) {
      throw Error('We do not have enough balance to fund the channel')
    }
    return this.chain.fundChannel(
      this.publicKey.to_address(),
      dest.to_address(),
      myFund,
      counterpartyFund,
      (txHash: string) => this.setTxHandler(`channel-updated-${txHash}`, txHash)
    )
  }

  /**
   * Checks whether a given `hoprNode` is allowed access.
   * When the register is disabled, a `hoprNode` is seen as `registered`,
   * when the register is enabled, a `hoprNode` needs to also be `eligible`.
   * @param hoprNode the public key of the account we want to check if it's registered
   * @returns true if registered
   */
  public async isAllowedAccessToNetwork(hoprNode: PublicKey): Promise<boolean> {
    try {
      // if register is disabled, all nodes are seen as "allowed"
      const registerEnabled = await this.db.is_network_registry_enabled()
      if (!registerEnabled) return true
      // find hoprNode's linked account
      const account = await this.db.get_account_from_network_registry(hoprNode)
      // check if account is eligible
      return this.db.is_eligible(account)
    } catch (error) {
      // log unexpected error
      if (!error?.notFound) log('error: could not determine whether node is is allowed access', error)
      return false
    }
  }

  public static createMockInstance(peer: PeerId): HoprCoreEthereum {
    const connectorLogger = debug(`hopr:mocks:connector`)
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
        return PublicKey.from_peerid_str(peer.toString())
      },
      getAccount: () => {
        connectorLogger('getAccount method was called')
        return Promise.resolve(
          new AccountEntry(
            PublicKey.from_peerid_str(peer.toString()),
            `/ip4/127.0.0.1/tcp/124/p2p/${peer.toString()}`,
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

export { useFixtures } from './indexer/index.mock.js'
export { sampleChainOptions } from './ethereum.mock.js'

export {
  ChannelEntry,
  ChannelCommitmentInfo,
  Indexer,
  ChainWrapper,
  createChainWrapper,
  initializeCommitment,
  findCommitmentPreImage,
  bumpCommitment
}
