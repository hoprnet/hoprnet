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
import { RoutingChannel } from './indexer'
import { PROVIDER_DEFAULT_URI, INDEXER_MAX_CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel } from './channel'
import { createChainWrapper } from './ethereum'
import { PROVIDER_CACHE_TTL } from './constants'

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

export default class HoprEthereum {
  // @TODO find a better solution
  private setCommitments: Map<string, boolean>
  private privateKey: Uint8Array
  private publicKey: PublicKey
  private address: Address

  constructor(private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    this.privateKey = this.chain.getPrivateKey()
    this.publicKey = this.chain.getPublicKey()
    this.address = Address.fromString(this.chain.getWallet().address)

    this.setCommitments = new Map()
    this.indexer.on('own-channel-updated', this.setInitialCommitmentIfNotSet.bind(this))
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector..')
    await this.indexer.stop()
  }

  public getChannel(src: PublicKey, counterparty: PublicKey) {
    return new Channel(src, counterparty, this.db, this.chain, this.indexer, this.privateKey)
  }

  async announce(multiaddr: Multiaddr): Promise<string> {
    return this.chain.announce(multiaddr)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    return this.chain.withdraw(currency, recipient, amount)
  }

  public getChannelsFromPeer(p: PeerId) {
    return this.indexer.getChannelsFromPeer(p)
  }

  public getChannelsOf(addr: Address): Promise<ChannelEntry[]> {
    return this.indexer.getChannelsOf(addr)
  }

  public async getAccount(addr: Address) {
    return this.indexer.getAccount(addr)
  }

  public getPublicKeyOf(addr: Address) {
    return this.indexer.getPublicKeyOf(addr)
  }

  public getRandomChannel() {
    return this.indexer.getRandomChannel()
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

  public smartContractInfo(): string {
    return this.chain.getInfo()
  }

  public async waitForPublicNodes(): Promise<Multiaddr[]> {
    return await this.indexer.getPublicNodes()
  }

  private async setInitialCommitmentIfNotSet(channel: ChannelEntry): Promise<void> {
    if (!this.address.eq(channel.partyA) && !this.address.eq(channel.partyB)) {
      return
    }

    const isPartyA = this.address.eq(channel.partyA)
    const counterparty = isPartyA ? channel.partyB : channel.partyA

    const alreadySet = this.setCommitments.get(Channel.generateId(this.address, counterparty).toHex())

    if (alreadySet != undefined) {
      log(`commitment already set, nothing to do`)
      return
    }

    this.setCommitments.set(Channel.generateId(this.address, counterparty).toHex(), true)

    if (!channel.ticketEpochFor(this.address).toBN().isZero()) {
      // Channel commitment is already set, nothing to do
      return
    }

    const counterpartyPubKey = await this.getPublicKeyOf(counterparty)

    return this.getChannel(this.publicKey, counterpartyPubKey).setCommitment()
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
      options.maxConfirmations ?? INDEXER_MAX_CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    await indexer.start()

    const ownChannelsWithoutCommitment = await indexer.getOwnChannelsWithoutCommitment()

    log('own channels without commitment', ownChannelsWithoutCommitment)
    const coreConnector = new HoprEthereum(chain, db, indexer)

    for (const ownChannelWithoutCommitment of ownChannelsWithoutCommitment) {
      await coreConnector.setInitialCommitmentIfNotSet(ownChannelWithoutCommitment)
    }

    log(`using blockchain address ${coreConnector.getAddress().toHex()}`)
    log(chalk.green('Connector started'))

    return coreConnector
  }
}

export { Channel, Indexer, RoutingChannel }
