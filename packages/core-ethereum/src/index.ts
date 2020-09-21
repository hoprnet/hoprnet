import type { Networks } from './ethereum/addresses'
import Web3 from 'web3'
import type { LevelUp } from 'levelup'
import HoprChannelsAbi from './ethereum/abi/HoprChannels.json'
import HoprTokenAbi from './ethereum/abi/HoprToken.json'
import type { Currencies } from '@hoprnet/hopr-core-connector-interface'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import chalk from 'chalk'
import { ChannelFactory } from './channel'
import types from './types'
import Tickets from './tickets'
import Indexer from './indexer'
import * as dbkeys from './dbKeys'
import * as utils from './utils'
import * as constants from './constants'
import * as config from './config'
import type { HoprChannels } from './tsc/web3/HoprChannels'
import type { HoprToken } from './tsc/web3/HoprToken'
import Account from './account'
import HashedSecret from './hashedSecret'
import Path from './path'

export default class HoprEthereum implements HoprCoreConnector {
  private _status: 'uninitialized' | 'initialized' | 'started' | 'stopped' = 'uninitialized'
  private _initializing?: Promise<void>
  private _starting?: Promise<void>
  private _stopping?: Promise<void>
  public signTransaction: ReturnType<typeof utils.TransactionSigner>
  public log: ReturnType<typeof utils['Log']>

  public channel: ChannelFactory
  public types: types
  public indexer: Indexer
  public account: Account
  public tickets: Tickets
  public hashedSecret: HashedSecret
  public path: Path

  constructor(
    public db: LevelUp,
    public web3: Web3,
    public chainId: number,
    public network: Networks,
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
    this.path = new Path(this)

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
        await this.account.stop()

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
    try {
      await this.hashedSecret.initialize(nonce)
    } catch (err) {
      this.log(chalk.red('Unable to submit secret'))
      this.log(chalk.red(err.message))
    }
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

  withdraw(currency: Currencies, recipient: string, amount: string): Promise<string> {
    return new Promise<string>(async (resolve, reject) => {
      try {
        if (currency === 'NATIVE') {
          this.web3.eth
            .sendTransaction({
              from: (await this.account.address).toHex(),
              to: recipient,
              nonce: await this.account.nonce,
              value: amount,
            })
            .on('transactionHash', (txHash) => resolve(txHash))
            .catch((err) => reject(err))
        } else {
          const tx = await this.signTransaction(this.hoprToken.methods.transfer(recipient, amount), {
            from: (await this.account.address).toHex(),
            to: this.hoprToken.options.address,
            nonce: await this.account.nonce,
          })

          tx.send()
          resolve(tx.transactionHash)
        }
      } catch (err) {
        reject(err)
      }
    })
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
    const network = utils.getNetworkName(chainId) as Networks

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
    coreConnector.log(`using blockchain address ${(await coreConnector.account.address).toHex()}`)

    const account = (await coreConnector.account.address).toHex()
    coreConnector.log(`using blockchain address ${account}`)

    if (+(await web3.eth.getBalance(account)) === 0) {
      throw Error(`account has no funds, please add some on ${account}`)
    }

    // begin initializing
    coreConnector.initialize().catch((err: Error) => {
      coreConnector.log(chalk.red(`coreConnector.initialize error: ${err.message}`))
    })
    coreConnector.start()

    return coreConnector
  }

  static get constants() {
    return constants
  }
}

export const Types = types
export const Utils = utils
