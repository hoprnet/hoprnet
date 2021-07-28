import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type { ChainWrapper } from './ethereum'
import chalk from 'chalk'
import debug from 'debug'
import {
  AcknowledgedTicket,
  PublicKey,
  Balance,
  Address,
  NativeBalance,
  cacheNoArgAsyncFunction,
  HoprDB,
  ChannelEntry
} from '@hoprnet/hopr-utils'
import Indexer from './indexer'
import { PROVIDER_DEFAULT_URI, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel } from './channel'
import { createChainWrapper } from './ethereum'
import { PROVIDER_CACHE_TTL } from './constants'
import { EventEmitter } from 'events'

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
  private privateKey: Uint8Array
  private publicKey: PublicKey
  private address: Address

  constructor(private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    super()
    this.privateKey = this.chain.getPrivateKey()
    this.publicKey = this.chain.getPublicKey()
    this.address = Address.fromString(this.chain.getWallet().address)
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
    return this.chain.announce(multiaddr)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    return this.chain.withdraw(currency, recipient, amount)
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

  private uncachedGetBalance = () => this.chain.getBalance(this.address)
  private cachedGetBalance = cacheNoArgAsyncFunction<Balance>(this.uncachedGetBalance, PROVIDER_CACHE_TTL)
  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetBalance() : this.uncachedGetBalance()
  }

  public getAddress(): Address {
    return this.address
  }

  public getPublicKey() {
    return this.publicKey
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => this.chain.getNativeBalance(this.address)
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

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param privateKey that is used to derive that on-chain identity
   * @param options.provider provider URI that is used to connect to the blockchain
   * @returns a promise resolved to the connector
   */
  public static async create(
    db: HoprDB,
    privateKey: Uint8Array,
    options?: { provider?: string; maxConfirmations?: number }
  ): Promise<HoprEthereum> {
    const chain = await createChainWrapper(options?.provider || PROVIDER_DEFAULT_URI, privateKey)
    await chain.waitUntilReady()

    const indexer = new Indexer(
      chain.getGenesisBlock(),
      db,
      chain,
      options.maxConfirmations ?? CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    await indexer.start()

    const coreConnector = new HoprEthereum(chain, db, indexer)

    log(`using blockchain address ${coreConnector.getAddress().toHex()}`)
    log(chalk.green('Connector started'))

    return coreConnector
  }
}

export { ChannelEntry, Channel, Indexer }
