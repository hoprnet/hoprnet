import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type { ChainWrapper } from './ethereum'
import chalk from 'chalk'
import { debug } from '@hoprnet/hopr-utils'
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
  Hash
} from '@hoprnet/hopr-utils'
import Indexer from './indexer'
import { PROVIDER_DEFAULT_URI, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel, redeemTickets } from './channel'
import { createChainWrapper } from './ethereum'
import { PROVIDER_CACHE_TTL } from './constants'
import { EventEmitter } from 'events'
import { initializeCommitment } from './commitment'

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

export default class HoprEthereum extends EventEmitter {
  public indexer: Indexer
  private chain: ChainWrapper
  private started: Promise<HoprEthereum> | undefined
  private redeemingAll: Promise<void> | undefined = undefined

  constructor(
    //private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    private db: HoprDB,
    private publicKey: PublicKey,
    private privateKey: Uint8Array,
    private options?: { provider?: string; maxConfirmations?: number }
  ) {
    super()
    this.indexer = new Indexer(
      this.publicKey.toAddress(),
      this.db,
      this.options.maxConfirmations ?? CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
  }

  async start(): Promise<HoprEthereum> {
    if (this.started) {
      return this.started
    }

    const _start = async (): Promise<HoprEthereum> => {
      this.chain = await createChainWrapper(this.options.provider || PROVIDER_DEFAULT_URI, this.privateKey)
      await this.chain.waitUntilReady()
      await this.indexer.start(this.chain, this.chain.getGenesisBlock())

      // Debug log used in e2e integration tests, please don't change
      log(`using blockchain address ${this.publicKey.toAddress().toHex()}`)
      log(chalk.green('Connector started'))
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

  public async getChannelTo(dest: PublicKey): Promise<ChannelEntry> {
    return await this.indexer.getChannel(generateChannelId(this.publicKey.toAddress(), dest.toAddress()))
  }

  async announce(multiaddr: Multiaddr): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    const tx = await this.chain.announce(multiaddr)
    // event emitted by the indexer
    return this.indexer.resolvePendingTransaction('announce', tx)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    // promise of tx hash gets resolved when the tx is mined.
    const tx = await this.chain.withdraw(currency, recipient, amount)
    // event emitted by the indexer
    return this.indexer.resolvePendingTransaction(currency === 'NATIVE' ? 'withdraw-native' : 'withdraw-hopr', tx)
  }

  public getOpenChannelsFrom(p: PublicKey) {
    return this.indexer.getOpenChannelsFrom(p)
  }

  public getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    return this.indexer.getChannelsFrom(addr)
  }

  public getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    return this.indexer.getChannelsTo(addr)
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

  private uncachedGetBalance = () => this.chain.getBalance(this.publicKey.toAddress())
  private cachedGetBalance = cacheNoArgAsyncFunction<Balance>(this.uncachedGetBalance, PROVIDER_CACHE_TTL)
  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetBalance() : this.uncachedGetBalance()
  }

  public getPublicKey() {
    return this.publicKey
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => this.chain.getNativeBalance(this.publicKey.toAddress())
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
      const tx = await this.chain.setCommitment(c.source.toAddress(), commitment)
      return this.indexer.resolvePendingTransaction('channel-updated', tx)
    }
    const getCommitment = async () => (await this.indexer.getChannel(c.getId())).commitment
    initializeCommitment(this.db, c.getId(), getCommitment, setCommitment)
  }

  public async redeemAllTickets(): Promise<void> {
    if (this.redeemingAll) {
      return this.redeemingAll
    }
    const _redeemAll = async () => {
      for (const ce of await this.getChannelsTo(this.publicKey.toAddress())) {
        await redeemTickets(ce.source, this.db, this.chain, this.indexer, this)
      }
      this.redeemingAll = undefined
    }
    this.redeemingAll = _redeemAll()
    return this.redeemingAll
  }

  public getPrivateKey(): Uint8Array {
    return this.privateKey
  }


  async initializeClosure(dest: PublicKey): Promise<string> { 
    const c = await this.getChannelTo(dest)
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    const tx = await this.chain.initiateChannelClosure(dest.toAddress())
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }

  public async finalizeClosure(dest: PublicKey): Promise<string> {
    const c = await this.getChannelTo(dest)
    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(dest.toAddress())
  }
}

export { ChannelEntry, Channel, Indexer, createChainWrapper, INDEXER_BLOCK_RANGE, CONFIRMATIONS }
