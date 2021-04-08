import type { LevelUp } from 'levelup'
import type { WebsocketProvider } from 'web3-core'
import type { Currencies, SubmitTicketResponse } from '@hoprnet/hopr-core-connector-interface'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { HoprChannels } from './tsc/web3/HoprChannels'
import type { HoprToken } from './tsc/web3/HoprToken'
import Web3 from 'web3'
import chalk from 'chalk'
import { Networks, getAddresses, abis } from '@hoprnet/hopr-ethereum'
import Channel from './channel'
import types from './types'
import { PublicKey } from './types'
import Indexer from './indexer'
import * as dbkeys from './dbKeys'
import * as utils from './utils'
import * as constants from './constants'
import * as config from './config'
import Account from './account'
import HashedSecret from './hashedSecret'
import debug from 'debug'
import { getWinProbabilityAsFloat, computeWinningProbability } from './utils'

const HoprChannelsAbi = abis.HoprChannels
const HoprTokenAbi = abis.HoprToken

const log = debug('hopr-core-ethereum')
let provider: WebsocketProvider

export default class HoprEthereum implements HoprCoreConnector {
  private _status: 'dead' | 'alive' = 'dead'
  private _starting?: Promise<HoprEthereum>
  private _stopping?: Promise<void>
  private _debug: boolean

  public channel = Channel
  public types: types
  public indexer: Indexer
  public account: Account
  public hashedSecret: HashedSecret

  constructor(
    public db: LevelUp,
    public web3: Web3,
    public chainId: number,
    public network: Networks,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken,
    debug: boolean,
    privateKey: Uint8Array,
    publicKey: PublicKey,
    maxConfirmations: number
  ) {
    this.account = new Account(this, privateKey, publicKey, chainId)
    this.indexer = new Indexer(this, maxConfirmations)
    this.types = new types()
    this._debug = debug
    this.hashedSecret = new HashedSecret(this.db, this.account, this.hoprChannels)
  }

  readonly dbKeys = dbkeys
  readonly constants = constants
  readonly CHAIN_NAME = 'HOPR on Ethereum'

  private async _start(): Promise<HoprEthereum> {
    await this.waitForWeb3()
    // await this.initOnchainValues()
    await this.indexer.start()
    await provider.connect()
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
        provider.disconnect(1000, 'Stopping HOPR node.')
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

  /**
   * Initializes the on-chain values of our account.
   */
  public async initOnchainValues(): Promise<void> {
    await this.hashedSecret.initialize(this._debug) // no-op if already initialized
  }

  /**
   * Checks whether web3 connection is alive
   * @returns a promise resolved true if web3 connection is alive
   */
  private async checkWeb3(): Promise<void> {
    if (!(await this.web3.eth.net.isListening())) {
      throw Error('web3 is not connected')
    }
  }

  // Web3's API leaves a lot to be desired...
  private async waitForWeb3(iterations: number = 0): Promise<void> {
    try {
      return await this.checkWeb3()
    } catch (e) {
      log('error when waiting for web3, try again', e)
      await utils.wait(1 * 1e3)
      if (iterations < 2) {
        this.waitForWeb3(iterations + 1)
      } else {
        throw new Error('giving up connecting to web3 after ' + iterations + 'attempts')
      }
    }
  }

  withdraw(currency: Currencies, recipient: string, amount: string): Promise<string> {
    return new Promise<string>(async (resolve, reject) => {
      try {
        if (currency === 'NATIVE') {
          const tx = await this.account.signTransaction({
            from: this.account.address.toHex(),
            to: recipient,
            value: amount
          })

          tx.send().once('transactionHash', (hash) => resolve(hash))
        } else {
          const tx = await this.account.signTransaction(
            {
              from: this.account.address.toHex(),
              to: this.hoprToken.options.address
            },
            this.hoprToken.methods.transfer(recipient, amount)
          )

          tx.send().once('transactionHash', (hash) => resolve(hash))
        }
      } catch (err) {
        reject(err)
      }
    })
  }

  public async hexAccountAddress(): Promise<string> {
    return this.account.address.toHex()
  }

  public smartContractInfo(): string {
    const network = utils.getNetworkName(this.chainId)
    const addr = getAddresses()[network]
    return [`Running on: ${network}`, `HOPR Token: ${addr.HoprToken}`, `HOPR Channels: ${addr.HoprChannels}`].join('\n')
  }

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param seed that is used to derive that on-chain identity
   * @param options.provider provider URI that is used to connect to the blockchain
   * @param options.debug debug mode, will generate account secrets using account's public key
   * @returns a promise resolved to the connector
   */
  public static async create(
    db: LevelUp,
    seed: Uint8Array,
    options?: { id?: number; provider?: string; debug?: boolean; maxConfirmations?: number }
  ): Promise<HoprEthereum> {
    const providerUri = options?.provider || config.DEFAULT_URI

    provider = new Web3.providers.WebsocketProvider(providerUri, {
      reconnect: {
        auto: true,
        delay: 1000, // ms
        maxAttempts: 30
      }
    })

    const web3 = new Web3(provider)
    const addresses = getAddresses()

    const chainId = await utils.getChainId(web3)
    const network = utils.getNetworkName(chainId) as Networks
    const publicKey = PublicKey.fromPrivKey(seed)

    if (typeof addresses?.[network]?.HoprChannels === 'undefined') {
      throw Error(`token contract address from network ${network} not found`)
    }

    const hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, addresses?.[network]?.HoprChannels)
    const hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.[network]?.HoprToken)

    const coreConnector = new HoprEthereum(
      db,
      web3,
      chainId,
      network,
      hoprChannels,
      hoprToken,
      options?.debug || false,
      seed,
      publicKey,
      options.maxConfirmations ?? config.MAX_CONFIRMATIONS
    )
    log(`using blockchain address ${await coreConnector.hexAccountAddress()}`)
    return coreConnector
  }

  static get constants() {
    return constants
  }
}

export * from './types'
export { Channel, SubmitTicketResponse, getWinProbabilityAsFloat, computeWinningProbability }
