import type { addresses } from '@hoprnet/hopr-ethereum'
import Web3 from 'web3'
import { LevelUp } from 'levelup'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import chalk from 'chalk'
import { ChannelFactory } from './channel'
import types from './types'
import Tickets from './tickets'
import Indexer from './indexer'
import * as dbkeys from './dbKeys'
import * as utils from './utils'
import * as constants from './constants'
import * as config from './config'
import { HoprChannels } from './tsc/web3/HoprChannels'
import { HoprToken } from './tsc/web3/HoprToken'
import Account from './account'
import HashedSecret from './hashedSecret'

export default class HoprEthereum implements HoprCoreConnector {
  private _status: 'uninitialized' | 'initialized' | 'started' | 'stopped' = 'uninitialized'
  private _initializing: Promise<void>
  private _starting: Promise<void>
  private _stopping: Promise<void>
  public signTransaction: ReturnType<typeof utils.TransactionSigner>
  public log: ReturnType<typeof utils['Log']>

  public channel: ChannelFactory
  public types: types
  public indexer: Indexer
  public account: Account
  public tickets: Tickets
  public hashedSecret: HashedSecret

  constructor(
    public db: LevelUp,
    public web3: Web3,
    public chainId: number,
    public network: addresses.Networks,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken,
    public options: {
      debug: boolean
    },
    privateKey: Uint8Array,
    publicKey: Uint8Array
  ) {
    this.hashedSecret = new HashedSecret(this)
    this.account = new Account(this, privateKey, publicKey)
    this.indexer = new Indexer(this)
    this.tickets = new Tickets(this)
    this.types = new types()
    this.channel = new ChannelFactory(this)

    this.signTransaction = utils.TransactionSigner(web3, privateKey)
    this.log = utils.Log()
  }

  readonly dbKeys = dbkeys
  readonly utils = utils
  readonly constants = constants
  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Initialises the connector, e.g. connect to a blockchain node.
   */
  async start() {
    this.log('Starting connector..')

    if (typeof this._starting !== 'undefined') {
      this.log('Connector is already starting..')
      return this._starting
    } else if (this._status === 'started') {
      this.log('Connector has already started')
      return
    } else if (this._status === 'uninitialized' && typeof this._initializing === 'undefined') {
      this.log('Connector was asked to start but state was not asked to initialize, initializing..')
      this.initialize().catch((err: Error) => {
        this.log(chalk.red(err.message))
      })
    }

    this._starting = Promise.resolve()
      .then(async () => {
        // agnostic check if connector can start
        while (this._status !== 'initialized') {
          await utils.wait(1 * 1e3)
        }

        // restart
        await this.indexer.start()

        this._status = 'started'
        this.log(chalk.green('Connector started'))
      })
      .catch((err: Error) => {
        this.log(chalk.red(`Connector failed to start: ${err.message}`))
      })
      .finally(() => {
        this._starting = undefined
      })

    return this._starting
  }

  /**
   * Stops the connector.
   */
  async stop(): Promise<void> {
    this.log('Stopping connector..')

    if (typeof this._stopping !== 'undefined') {
      this.log('Connector is already stopping..')
      return this._stopping
    } else if (this._status === 'stopped') {
      this.log('Connector has already stopped')
      return
    }

    this._stopping = Promise.resolve()
      .then(async () => {
        // connector is starting
        if (typeof this._starting !== 'undefined') {
          this.log("Connector will stop once it's started")
          // @TODO: cancel initializing & starting
          await this._starting
        }

        await this.indexer.stop()

        this._status = 'stopped'
        this.log(chalk.green('Connector stopped'))
      })
      .catch((err: Error) => {
        this.log(chalk.red(`Connector failed to stop: ${err.message}`))
      })
      .finally(() => {
        this._stopping = undefined
      })

    return this._stopping
  }

  get started() {
    return this._status === 'started'
  }

  /**
   * Initializes the on-chain values of our account.
   * @param nonce optional specify nonce of the account to run multiple queries simultaneously
   */
  async initOnchainValues(nonce?: number): Promise<void> {
    await this.hashedSecret.submit(nonce)
  }

  /**
   * Initializes connector, insures that connector is only initialized once,
   * and it only resolves once it's done initializing.
   */
  async initialize(): Promise<void> {
    this.log('Initializing connector..')

    if (typeof this._initializing !== 'undefined') {
      this.log('Connector is already initializing..')
      return this._initializing
    } else if (this._status === 'initialized') {
      this.log('Connector has already initialized')
      return
    } else if (this._status !== 'uninitialized') {
      throw Error(`invalid status '${this._status}', could not initialize`)
    }

    this._initializing = Promise.resolve()
      .then(async () => {
        // initialize stuff
        await Promise.all([
          // confirm web3 is connected
          this.checkWeb3(),
          // start channels indexing
          this.indexer.start(),
          // check account secret
          this.hashedSecret.check(),
        ])

        this._status = 'initialized'
        this.log(chalk.green('Connector initialized'))
      })
      .catch((err: Error) => {
        this.log(`Connector failed to initialize: ${err.message}`)
      })
      .finally(() => {
        this._initializing = undefined
      })

    return this._initializing
  }

  /**
   * Checks whether web3 connection is alive
   * @returns a promise resolved true if web3 connection is alive
   */
  async checkWeb3(): Promise<void> {
    let isListening
    try {
      isListening = await this.web3.eth.net.isListening()
    } catch (err) {
      this.log(chalk.red(`error checking web3: ${err.message}`))
    }

    if (!isListening) {
      throw Error('web3 is not connected')
    }
  }

  static get constants() {
    return constants
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
  static async create(
    db: LevelUp,
    seed: Uint8Array,
    options?: { id?: number; provider?: string; debug?: boolean }
  ): Promise<HoprEthereum> {
    const providerUri = options?.provider || config.DEFAULT_URI

    const provider = new Web3.providers.WebsocketProvider(providerUri, {
      reconnect: {
        auto: true,
        delay: 1000, // ms
        maxAttempts: 10,
      },
    })

    const web3 = new Web3(provider)

    const [chainId, publicKey] = await Promise.all([
      /* prettier-ignore */
      utils.getChainId(web3),
      utils.privKeyToPubKey(seed),
    ])
    const network = utils.getNetworkName(chainId)

    if (typeof config.CHANNELS_ADDRESSES[network] === 'undefined') {
      throw Error(`channel contract address from network ${network} not found`)
    }
    if (typeof config.TOKEN_ADDRESSES[network] === 'undefined') {
      throw Error(`token contract address from network ${network} not found`)
    }

    const hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, config.CHANNELS_ADDRESSES[network])
    const hoprToken = new web3.eth.Contract(HoprTokenAbi as any, config.TOKEN_ADDRESSES[network])

    const coreConnector = new HoprEthereum(
      db,
      web3,
      chainId,
      network,
      hoprChannels,
      hoprToken,
      { debug: options?.debug || false },
      seed,
      publicKey
    )
    coreConnector.log(`using ethereum address ${(await coreConnector.account.address).toHex()}`)

    // begin initializing
    coreConnector.initialize().catch((err: Error) => {
      coreConnector.log(chalk.red(`coreConnector.initialize error: ${err.message}`))
    })
    coreConnector.start()

    return coreConnector
  }
}

export const Types = types
export const Utils = utils
