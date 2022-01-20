import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import { ChainWrapper, createChainWrapper } from './ethereum'
import chalk from 'chalk'
import {
  AcknowledgedTicket,
  PublicKey,
  Balance,
  Address,
  NativeBalance,
  cacheNoArgAsyncFunction,
  HoprDB,
  ChannelEntry,
  ChannelStatus,
  generateChannelId,
  Hash,
  debug,
  DeferType,
  privKeyToPeerId
} from '@hoprnet/hopr-utils'
import Indexer from './indexer'
import { CONFIRMATIONS, INDEXER_BLOCK_RANGE, PROVIDER_CACHE_TTL } from './constants'
import { EventEmitter } from 'events'
import { initializeCommitment, findCommitmentPreImage, bumpCommitment, ChannelCommitmentInfo } from './commitment'
import { IndexerEvents } from './indexer/types'
import ChainWrapperSingleton from './chain'

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
  gasPrice?: number
  network: string
  environment: string
}

export default class HoprCoreEthereum extends EventEmitter {
  public indexer: Indexer
  private chain: ChainWrapper
  private started: Promise<HoprCoreEthereum> | undefined
  private redeemingAll: Promise<void> | undefined = undefined

  constructor(
    //private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    private db: HoprDB,
    private publicKey: PublicKey,
    private privateKey: Uint8Array,
    private options: ChainOptions,
    protected automaticChainCreation = true
  ) {
    super()
    this.indexer = new Indexer(
      this.publicKey.toAddress(),
      this.db,
      this.options?.maxConfirmations ?? CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    // In some cases, we want to make sure the chain within the connector is not triggered
    // automatically but instead via an event. This is the case for `hoprd`, where we need
    // to get notified after ther chain was properly created, and we can't get setup the
    // listeners before the node was actually created.
    if (automaticChainCreation) {
      this.createChain()
    } else {
      this.once('connector:create', this.createChain)
    }
  }

  private async createChain(): Promise<void> {
    try {
      this.chain = await ChainWrapperSingleton.create(this.options, this.privateKey)
      // Emit event to make sure connector is aware the chain was created properly.
      this.emit('connector:created')
    } catch (err) {
      log('error: failed to create provider chain wrapper', err)
    }
  }

  async start(): Promise<HoprCoreEthereum> {
    if (this.started) {
      return this.started
    }

    const _start = async (): Promise<HoprCoreEthereum> => {
      try {
        await this.chain.waitUntilReady()

        const hoprBalance = await this.chain.getBalance(this.publicKey.toAddress())
        await this.db.setHoprBalance(hoprBalance)
        log(`set HOPR balance to ${hoprBalance.toFormattedString()}`)

        await this.indexer.start(this.chain, this.chain.getGenesisBlock())

        // Debug log used in e2e integration tests, please don't change
        log(`using blockchain address ${this.publicKey.toAddress().toHex()}`)
        log(chalk.green('Connector started'))
      } catch (err) {
        log('error: failed to start the indexer', err)
      }
      return this
    }
    this.started = _start()
    return this.started
  }

  public getChain(): ChainWrapper {
    return this.chain
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector...')
    this.indexer.stop()
  }

  async announce(multiaddr: Multiaddr): Promise<string> {
    return this.chain.announce(multiaddr, (tx: string) => this.setTxHandler('announce', tx))
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    return this.chain.withdraw(currency, recipient, amount, (tx: string) =>
      this.setTxHandler(currency === 'NATIVE' ? 'withdraw-native' : 'withdraw-hopr', tx)
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
    return useIndexer ? this.db.getHoprBalance() : this.chain.getBalance(this.publicKey.toAddress())
  }

  public getPublicKey(): PublicKey {
    return this.publicKey
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => {
    log('Chain [inside cached hopr-ethereum]', this.chain)
    return this.chain.getNativeBalance(this.publicKey.toAddress())
  }
  private cachedGetNativeBalance = cacheNoArgAsyncFunction<NativeBalance>(
    this.uncachedGetNativeBalance,
    PROVIDER_CACHE_TTL
  )
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    return useCache ? this.cachedGetNativeBalance() : this.uncachedGetNativeBalance()
  }

  public smartContractInfo(): {
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    channelClosureSecs: number
  } {
    return this.chain.getInfo()
  }

  public async waitForPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return await this.indexer.getPublicNodes()
  }

  public async commitToChannel(c: ChannelEntry): Promise<void> {
    log('committing to channel', c)
    const setCommitment = async (commitment: Hash) => {
      return this.chain.setCommitment(c.source.toAddress(), commitment, (tx: string) =>
        this.setTxHandler('channel-updated', tx)
      )
    }
    const getCommitment = async () => (await this.db.getChannel(c.getId())).commitment

    // Get all channel information required to build the initial commitment
    const cci = new ChannelCommitmentInfo(
      this.options.chainId,
      this.smartContractInfo().hoprChannelsAddress,
      c.getId(),
      c.channelEpoch
    )

    await initializeCommitment(this.db, privKeyToPeerId(this.privateKey), cci, getCommitment, setCommitment)
  }

  public async redeemAllTickets(): Promise<void> {
    if (this.redeemingAll) {
      return this.redeemingAll
    }
    const _redeemAll = async () => {
      for (const ce of await this.db.getChannelsTo(this.publicKey.toAddress())) {
        await this.redeemTicketsInChannel(ce)
      }
      this.redeemingAll = undefined
    }
    this.redeemingAll = _redeemAll()
    return this.redeemingAll
  }

  private async redeemTicketsInChannel(channel: ChannelEntry) {
    if (!channel.destination.eq(this.getPublicKey())) {
      throw new Error('Cannot redeem ticket in channel that isnt to us')
    }
    // Because tickets are ordered and require the previous redemption to
    // have succeeded before we can redeem the next, we need to do this
    // sequentially.
    const tickets = await this.db.getAcknowledgedTickets({ channel })
    log(`redeeming ${tickets.length} tickets from ${channel.source.toB58String()}`)
    try {
      for (const ticket of tickets) {
        log('redeeming ticket', ticket)
        const result = await this.redeemTicket(channel.source, ticket)
        if (result.status !== 'SUCCESS') {
          log('Error redeeming ticket', result)
          // We need to abort as tickets require ordered redemption.
          return
        }
        log('ticket was redeemed')
      }
    } catch (e) {
      // We are going to swallow the error here, as more than one consumer may
      // be inspecting this same promise.
      log('Error when redeeming tickets, aborting', e)
    }
    log(`redemption of tickets from ${channel.source.toB58String()} is complete`)
  }

  // Private as out of order redemption will break things - redeem all at once.
  private async redeemTicket(counterparty: PublicKey, ackTicket: AcknowledgedTicket): Promise<RedeemTicketResponse> {
    if (!ackTicket.verify(counterparty)) {
      return {
        status: 'FAILURE',
        message: 'Invalid response to acknowledgement'
      }
    }

    try {
      const ticket = ackTicket.ticket

      log('Submitting ticket', ackTicket.response.toHex())
      const emptyPreImage = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
      const hasPreImage = !ackTicket.preImage.eq(emptyPreImage)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message: 'PreImage is empty.'
        }
      }

      const isWinning = ticket.isWinningTicket(ackTicket.preImage, ackTicket.response, ticket.winProb)

      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const receipt = await this.chain.redeemTicket(counterparty.toAddress(), ackTicket, ticket, (tx: string) =>
        this.setTxHandler('channel-updated', tx)
      )

      log('Successfully submitted ticket', ackTicket.response.toHex())
      await this.db.markRedeemeed(ackTicket)
      this.emit('ticket:redeemed', ackTicket)
      return {
        status: 'SUCCESS',
        receipt,
        ackTicket
      }
    } catch (err) {
      // TODO delete ackTicket -- check if it's due to gas!
      log('Unexpected error when redeeming ticket', ackTicket.response.toHex(), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
  }

  async initializeClosure(dest: PublicKey): Promise<string> {
    const c = await this.db.getChannelTo(dest)
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    return this.chain.initiateChannelClosure(dest.toAddress(), (tx: string) => this.setTxHandler('channel-updated', tx))
  }

  public async finalizeClosure(dest: PublicKey): Promise<string> {
    const c = await this.db.getChannelTo(dest)
    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(dest.toAddress(), (tx: string) =>
      this.setTxHandler('channel-updated', tx)
    )
  }

  public async openChannel(dest: PublicKey, amount: Balance): Promise<Hash> {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.db.getChannelTo(dest)
    } catch {}
    if (c && c.status !== ChannelStatus.Closed) {
      throw Error('Channel is already opened')
    }

    const myBalance = await this.getBalance()
    if (myBalance.lt(amount)) {
      throw Error('We do not have enough balance to open a channel')
    }
    await this.chain.openChannel(this.publicKey.toAddress(), dest.toAddress(), amount, (tx: string) =>
      this.setTxHandler('channel-updated', tx)
    )
    return generateChannelId(this.publicKey.toAddress(), dest.toAddress())
  }

  public async fundChannel(dest: PublicKey, myFund: Balance, counterpartyFund: Balance) {
    const totalFund = myFund.add(counterpartyFund)
    const myBalance = await this.getBalance()
    if (totalFund.gt(myBalance)) {
      throw Error('We do not have enough balance to fund the channel')
    }
    return this.chain.fundChannel(
      this.publicKey.toAddress(),
      dest.toAddress(),
      myFund,
      counterpartyFund,
      (tx: string) => this.setTxHandler('channel-updated', tx)
    )
  }
}

export { createConnectorMock } from './index.mock'
export { useFixtures } from './indexer/index.mock'
export { sampleChainOptions } from './ethereum.mock'

export {
  ChannelEntry,
  ChannelCommitmentInfo,
  Indexer,
  ChainWrapperSingleton,
  ChainWrapper,
  createChainWrapper,
  initializeCommitment,
  findCommitmentPreImage,
  bumpCommitment,
  INDEXER_BLOCK_RANGE,
  CONFIRMATIONS
}
