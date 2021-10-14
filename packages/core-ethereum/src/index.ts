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
  Hash
} from '@hoprnet/hopr-utils'
import Indexer from './indexer'
import { PROVIDER_DEFAULT_URI, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel } from './channel'
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

  public getChannel(src: PublicKey, counterparty: PublicKey) {
    return new Channel(src, counterparty, this.db, this.chain, this.indexer, this.privateKey, this)
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
    const setCommitment = async (commitment: Hash) => this.chain.setCommitment(c.source.toAddress(), commitment).then(tx => this.indexer.resolvePendingTransaction('channel-updated', tx))
    const getCommitment = async () => (await this.indexer.getChannel(c.getId())).commitment
    initializeCommitment(this.db, c.getId(), getCommitment, setCommitment)
  }
}

export { ChannelEntry, Channel, Indexer, createChainWrapper, INDEXER_BLOCK_RANGE, CONFIRMATIONS }
