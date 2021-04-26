import type { LevelUp } from 'levelup'
import chalk from 'chalk'
import debug from 'debug'
import { Acknowledgement, PublicKey, Balance, Address, NativeBalance } from './types'
import Indexer from './indexer'
import { RoutingChannel } from './indexer'
import { getWinProbabilityAsFloat, computeWinningProbability } from './utils'
import { DEFAULT_URI, MAX_CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel } from './channel'
import { createChainWrapper } from './ethereum'
import type { ChainWrapper } from './ethereum'
import type PeerId from 'peer-id'
import { PROVIDER_CACHE_TTL } from './constants'
import { isExpired } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

const log = debug('hopr-core-ethereum')

export type SubmitTicketResponse =
  | {
      status: 'SUCCESS'
      receipt: string
      ackTicket: Acknowledgement
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
  private _status: 'dead' | 'alive' = 'dead'
  private _starting?: Promise<HoprEthereum>
  private _stopping?: Promise<void>
  private indexer: Indexer
  private balanceCache = new Map<'balance' | 'nativeBalance', { value: string; updatedAt: number }>()
  private privateKey: Uint8Array

  constructor(private chain: ChainWrapper, private db: LevelUp, maxConfirmations: number, blockRange: number) {
    this.indexer = new Indexer(chain.getGenesisBlock(), this.db, this.chain, maxConfirmations, blockRange)
    this.privateKey = this.chain.getPrivateKey()
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  private async _start(): Promise<HoprEthereum> {
    await this.chain.waitUntilReady()
    await this.indexer.start()
    this._status = 'alive'
    log(chalk.green('Connector started'))
    return this
  }

  /**
   * Initialises the connector, e.g. connect to a blockchain node.
   */
  public async start(): Promise<HoprEthereum> {
    log('Starting connector..')
    if (this._status === 'alive') {
      log('Connector has already started')
      return Promise.resolve(this)
    }
    if (!this._starting) {
      this._starting = this._start()
      this._starting.finally(() => {
        this._starting = undefined
      })
    }
    return this._starting
  }

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    log('Stopping connector..')
    if (typeof this._stopping !== 'undefined') {
      return this._stopping
    } else if (this._status === 'dead') {
      return
    }

    this._stopping = Promise.resolve()
      .then(async () => {
        if (this._starting) {
          log("Connector will stop once it's started")
          await this._starting
        }

        await this.indexer.stop()
        this._status = 'dead'
        log(chalk.green('Connector stopped'))
      })
      .finally(() => {
        this._stopping = undefined
      })
    return this._stopping
  }

  get started() {
    return this._status === 'alive'
  }

  public getChannel(src: PublicKey, counterparty: PublicKey) {
    return new Channel(src, counterparty, this.db, this.chain, this.indexer, this.privateKey)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    return this.chain.withdraw(currency, recipient, amount)
  }

  public async hexAccountAddress(): Promise<string> {
    return this.getAddress().toHex()
  }

  public getChannelsFromPeer(p: PeerId) {
    return this.indexer.getChannelsFromPeer(p)
  }

  public getRandomChannel() {
    return this.indexer.getRandomChannel()
  }

  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    if (useCache) {
      const cached = this.balanceCache.get('balance')
      const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
      if (notExpired) return new Balance(new BN(cached.value))
    }

    const value = await this.chain.getBalance(this.getAddress())
    this.balanceCache.set('balance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

    return value
  }

  getAddress(): Address {
    return Address.fromString(this.chain.getWallet().address)
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    if (useCache) {
      const cached = this.balanceCache.get('nativeBalance')
      const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
      if (notExpired) return new NativeBalance(new BN(cached.value))
    }

    const value = await this.chain.getNativeBalance(this.getAddress())
    this.balanceCache.set('nativeBalance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

    return value
  }
  /*
  public smartContractInfo(): string {
    const network = utils.getNetworkName(this.chainId)
    const contracts = getContracts()[network]
    return [
      `Running on: ${network}`,
      `HOPR Token: ${contracts.HoprToken.address}`,
      `HOPR Channels: ${contracts.HoprChannels.address}`
    ].join('\n')
  }
  */

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param privateKey that is used to derive that on-chain identity
   * @param options.provider provider URI that is used to connect to the blockchain
   * @returns a promise resolved to the connector
   */
  public static async create(
    db: LevelUp,
    privateKey: Uint8Array,
    options?: { provider?: string; maxConfirmations?: number }
  ): Promise<HoprEthereum> {
    const chain = await createChainWrapper(options?.provider || DEFAULT_URI, privateKey)
    const coreConnector = new HoprEthereum(
      chain,
      db,
      options.maxConfirmations ?? MAX_CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    log(`using blockchain address ${await coreConnector.hexAccountAddress()}`)
    return coreConnector
  }
}

export * from './types'
export { Channel, getWinProbabilityAsFloat, computeWinningProbability, Indexer, RoutingChannel }
