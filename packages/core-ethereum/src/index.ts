import type { LevelUp } from 'levelup'
import chalk from 'chalk'
import debug from 'debug'
import { Acknowledgement, PublicKey } from './types'
import Indexer from './indexer'
import { RoutingChannel } from './indexer'
import Account from './account'
import { getWinProbabilityAsFloat, computeWinningProbability } from './utils'
import { DEFAULT_URI, MAX_CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { Channel } from './channel'
import { createChainWrapper } from './ethereum'
import type { ChainWrapper } from './ethereum'

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

  public indexer: Indexer
  public account: Account

  constructor(private chain: ChainWrapper, private db: LevelUp, maxConfirmations: number, blockRange: number) {
    this.indexer = new Indexer(chain.getGenesisBlock(), this.db, this.chain, maxConfirmations, blockRange)
    this.account = new Account(this.chain, this.indexer, chain.getWallet())
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
    return new Channel(src, counterparty, this.db, this.chain, this.indexer, this.account.privateKey)
  }

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    return this.chain.withdraw(currency, recipient, amount)
  }

  public async hexAccountAddress(): Promise<string> {
    return this.account.getAddress().toHex()
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
