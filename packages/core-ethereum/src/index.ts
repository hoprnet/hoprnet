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
  private privateKey: Uint8Array

  constructor(private chain: ChainWrapper, private db: HoprDB, public indexer: Indexer) {
    this.privateKey = this.chain.getPrivateKey()
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

  public getChannelsOf(addr: Address) {
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

  private uncachedGetBalance = () => this.chain.getBalance(this.getAddress())
  private cachedGetBalance = cacheNoArgAsyncFunction<Balance>(this.uncachedGetBalance, PROVIDER_CACHE_TTL)
  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return useCache ? this.cachedGetBalance() : this.uncachedGetBalance()
  }

  getAddress(): Address {
    return Address.fromString(this.chain.getWallet().address)
  }

  getPublicKey() {
    return this.chain.getPublicKey()
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  private uncachedGetNativeBalance = () => this.chain.getNativeBalance(this.getAddress())
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

  public onOwnChannel(entry: ChannelEntry): Promise<void> {
    return onOwnChannel(this.chain.getPublicKey(), entry, this.getChannel.bind(this), this.getPublicKeyOf.bind(this))
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

    const self = chain.getPublicKey()

    const ownChannels: ChannelEntry[] = []
    const indexer = new Indexer(
      chain.getGenesisBlock(),
      db,
      chain,
      options.maxConfirmations ?? INDEXER_MAX_CONFIRMATIONS,
      INDEXER_BLOCK_RANGE,
      self.toAddress(),
      (channel: ChannelEntry) => ownChannels.push(channel)
    )
    await indexer.start()

    const coreConnector = new HoprEthereum(chain, db, indexer)
    log(`using blockchain address ${coreConnector.getAddress().toHex()}`)
    log(chalk.green('Connector started'))

    indexer.setOnOwnChannel(coreConnector.onOwnChannel.bind(coreConnector))

    for (const ownChannel of ownChannels) {
      await onOwnChannel(
        self,
        ownChannel,
        coreConnector.getChannel.bind(coreConnector),
        coreConnector.getPublicKeyOf.bind(coreConnector)
      )
    }

    return coreConnector
  }
}

async function onOwnChannel(
  self: PublicKey,
  entry: ChannelEntry,
  getChannel: InstanceType<typeof HoprEthereum>['getChannel'],
  getPublicKeyOf: (addr: Address) => Promise<PublicKey>
) {
  const isPartyA = self.toAddress().eq(entry.partyA)

  if (
    (isPartyA && !entry.partyATicketEpoch.toBN().isZero()) ||
    (!isPartyA && !entry.partyBTicketEpoch.toBN().isZero())
  ) {
    return
  }

  let channel: Channel
  if (self.toAddress().eq(entry.partyA)) {
    channel = getChannel(self, await getPublicKeyOf(entry.partyB))
  } else {
    channel = getChannel(await getPublicKeyOf(entry.partyB), self)
  }
  await channel.initCommitment()
}

export { Channel, Indexer, RoutingChannel }
