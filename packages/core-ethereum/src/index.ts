import type { LevelUp } from 'levelup'
import type { Wallet as IWallet, providers as IProviders } from 'ethers'
import type { HoprToken, HoprChannels } from './contracts'
import chalk from 'chalk'
import { Networks, getContracts } from '@hoprnet/hopr-ethereum'
import { ethers } from 'ethers'
import debug from 'debug'
import { Acknowledgement } from './types'
import Indexer from './indexer'
import { RoutingChannel } from './indexer'
import * as utils from './utils'
import Account from './account'
import { getWinProbabilityAsFloat, computeWinningProbability } from './utils'
import { HoprToken__factory, HoprChannels__factory } from './contracts'
import { DEFAULT_URI, MAX_CONFIRMATIONS, INDEXER_BLOCK_RANGE } from './constants'
import { ChainInteractions } from './chainInteractions'

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
  private chain

  public indexer: Indexer
  public account: Account

  constructor(
    public db: LevelUp,
    public provider: IProviders.WebSocketProvider,
    public chainId: number,
    public network: Networks,
    public wallet: IWallet,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken,
    genesisBlock: number,
    maxConfirmations: number,
    blockRange: number
  ) {
    this.indexer = new Indexer(this, genesisBlock, maxConfirmations, blockRange)
    this.chain = new ChainInteractions(this.provider, this.hoprToken)
    this.account = new Account(this.network, this.chain, this.indexer, this.wallet)
  }

  readonly CHAIN_NAME = 'HOPR on Ethereum'

  private async _start(): Promise<HoprEthereum> {
    await this.provider.ready
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

  async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    if (currency === 'NATIVE') {
      const nonceLock = await this.account.getNonceLock()
      try {
        const transaction = await this.account.wallet.sendTransaction({
          to: recipient,
          value: ethers.BigNumber.from(amount),
          nonce: ethers.BigNumber.from(nonceLock.nextNonce)
        })
        nonceLock.releaseLock()
        return transaction.hash
      } catch (err) {
        nonceLock.releaseLock()
        throw err
      }
    } else {
      const transaction = await this.account.sendTransaction(this.hoprToken.transfer, recipient, amount)
      return transaction.hash
    }
  }

  public async hexAccountAddress(): Promise<string> {
    return this.account.getAddress().toHex()
  }

  public smartContractInfo(): string {
    const network = utils.getNetworkName(this.chainId)
    const contracts = getContracts()[network]
    return [
      `Running on: ${network}`,
      `HOPR Token: ${contracts.HoprToken.address}`,
      `HOPR Channels: ${contracts.HoprChannels.address}`
    ].join('\n')
  }

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param privateKey that is used to derive that on-chain identity
   * @param options.provider provider URI that is used to connect to the blockchain
   * @param options.debug debug mode, will generate account secrets using account's public key
   * @returns a promise resolved to the connector
   */
  public static async create(
    db: LevelUp,
    privateKey: Uint8Array,
    options?: { id?: number; provider?: string; debug?: boolean; maxConfirmations?: number }
  ): Promise<HoprEthereum> {
    const provider = new ethers.providers.WebSocketProvider(options?.provider || DEFAULT_URI)
    const wallet = new ethers.Wallet(privateKey).connect(provider)
    const chainId = await provider.getNetwork().then((res) => res.chainId)
    const network = utils.getNetworkName(chainId) as Networks
    const contracts = getContracts()?.[network]

    if (!contracts?.HoprToken?.address) {
      throw Error(`token contract address from network ${network} not found`)
    } else if (!contracts?.HoprChannels?.address) {
      throw Error(`channels contract address from network ${network} not found`)
    }

    const hoprChannels = HoprChannels__factory.connect(contracts.HoprChannels.address, wallet)
    const hoprToken = HoprToken__factory.connect(contracts.HoprToken.address, wallet)

    const coreConnector = new HoprEthereum(
      db,
      provider,
      chainId,
      network,
      wallet,
      hoprChannels,
      hoprToken,
      contracts?.HoprChannels?.deployedAt ?? 0,
      options.maxConfirmations ?? MAX_CONFIRMATIONS,
      INDEXER_BLOCK_RANGE
    )
    log(`using blockchain address ${await coreConnector.hexAccountAddress()}`)
    return coreConnector
  }
}

export * from './types'
export { getWinProbabilityAsFloat, computeWinningProbability, Indexer, RoutingChannel }
