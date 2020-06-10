import type { addresses } from '@hoprnet/hopr-ethereum'
import { randomBytes, createHash } from 'crypto'
import Web3 from 'web3'
import { LevelUp } from 'levelup'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
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

export default class HoprEthereum implements HoprCoreConnector {
  private _status: 'uninitialized' | 'initialized' | 'started' | 'stopped' = 'uninitialized'
  private _initializing: Promise<void>
  public _onChainValuesInitialized: boolean
  private _starting: Promise<void>
  private _stopping: Promise<void>
  public signTransaction: ReturnType<typeof utils.TransactionSigner>
  public log: ReturnType<typeof utils['Log']>

  public channel: ChannelFactory
  public types: types
  public indexer: Indexer
  public account: Account
  public tickets: Tickets

  constructor(
    public db: LevelUp,
    public web3: Web3,
    public network: addresses.Networks,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken,
    public options: {
      debug: boolean
    },
    privateKey: Uint8Array,
    publicKey: Uint8Array
  ) {
    this.account = new Account(this, privateKey, publicKey)
    this.indexer = new Indexer(this)
    this.tickets = new Tickets(this)
    this.types = new types()
    this.channel = new ChannelFactory(this)

    this._onChainValuesInitialized = false
    this.signTransaction = utils.TransactionSigner(web3, privateKey)
    this.log = utils.Log()
  }

  readonly dbKeys = dbkeys
  readonly utils = utils
  readonly constants = constants
  readonly CHAIN_NAME = 'HOPR on Ethereum'

  /**
   * Returns the current balances of the account associated with this node (HOPR)
   * @returns a promise resolved to Balance
   */

  /**
   * Returns the current native balance (ETH)
   * @returns a promise resolved to Balance
   */

  // get ticketEpoch(): Promise<TicketEpoch> {}

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
    if (this._onChainValuesInitialized) {
      return
    }

    await this.setAccountSecret(nonce)

    this._onChainValuesInitialized = true
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
          this.checkAccountSecret(),
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
   * Checks whether node has an account secret set onchain and offchain
   * @returns a promise resolved true if secret is set correctly
   */
  async checkAccountSecret(): Promise<void> {
    let [onChainSecret, offChainSecret] = await Promise.all([
      // get onChainSecret
      this.hoprChannels.methods
        .accounts((await this.account.address).toHex())
        .call()
        .then((res) => stringToU8a(res.hashedSecret))
        .then((secret: Uint8Array) => {
          if (u8aEquals(secret, new Uint8Array(this.types.Hash.SIZE).fill(0x00))) {
            return undefined
          }

          return secret
        }),
      // get offChainSecret
      this.db.get(Buffer.from(dbkeys.OnChainSecret())).catch((err) => {
        if (err.notFound != true) {
          throw err
        }
      }),
    ])

    // @TODO check with most recent exponent and fail if it is not equal
    // if (!u8aEquals(onChainSecret, offChainSecret)) {
    //   throw Error(`Inconsistency found. On-chain secret is set to ${u8aToHex(onChainSecret)} whilst off-chain secret is  ${u8aToHex(offChainSecret)}`)
    // }

    let hasOffChainSecret = typeof offChainSecret !== 'undefined'
    let hasOnChainSecret = typeof onChainSecret !== 'undefined'

    if (hasOffChainSecret !== hasOnChainSecret) {
      if (hasOffChainSecret) {
        this.log(`Key is present off-chain but not on-chain, submitting..`)
        // @TODO this potentially dangerous because it increases the account counter
        await utils.waitForConfirmation(
          (
            await this.signTransaction(this.hoprChannels.methods.setHashedSecret(u8aToHex(offChainSecret)), {
              from: (await this.account.address).toHex(),
              to: this.hoprChannels.options.address,
              nonce: await this.account.nonce,
            })
          ).send()
        )
        hasOnChainSecret = true
      } else {
        this.log(`Key is present on-chain but not in our database.`)
        if (this.options.debug) {
          await this.db.put(Buffer.from(dbkeys.OnChainSecret()), Buffer.from(this.getDebugAccountSecret()))
          hasOffChainSecret = true
        } else {
          throw Error(`Key is present on-chain but not in our database.`)
        }
      }
    }

    this._onChainValuesInitialized = hasOffChainSecret && hasOnChainSecret
  }

  /**
   * generate and set account secret
   */
  async setAccountSecret(nonce?: number): Promise<void> {
    let secret: Uint8Array
    if (this.options.debug) {
      secret = this.getDebugAccountSecret()
    } else {
      secret = new Uint8Array(randomBytes(32))
    }

    const dbPromise = this.db.put(Buffer.from(this.dbKeys.OnChainSecret()), Buffer.from(secret.slice()))

    for (let i = 0; i < 500; i++) {
      secret = await this.utils.hash(secret)
    }

    await Promise.all([
      await utils.waitForConfirmation(
        (
          await this.signTransaction(this.hoprChannels.methods.setHashedSecret(u8aToHex(secret)), {
            from: (await this.account.address).toHex(),
            to: this.hoprChannels.options.address,
            nonce: nonce || (await this.account.nonce),
          })
        ).send()
      ),
      dbPromise,
    ])
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

  private getDebugAccountSecret(): Uint8Array {
    return createHash('sha256').update(this.account.keys.onChain.pubKey).digest()
  }

  static get constants() {
    return constants
  }

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param seed that is used to derive that on-chain identity
   * @param options.id Id of the demo account
   * @param options.provider provider URI that is used to connect to the blockchain
   * @param options.debug debug mode, will generate account secrets using account's public key
   * @returns a promise resolved to the connector
   */
  static async create(
    db: LevelUp,
    seed?: Uint8Array,
    options?: { id?: number; provider?: string; debug?: boolean }
  ): Promise<HoprEthereum> {
    const usingSeed = typeof seed !== 'undefined'
    const usingOptions = typeof options !== 'undefined'

    if (!usingSeed && !usingOptions) {
      throw Error("'seed' or 'options' must be provided")
    }
    if (usingOptions && typeof options.id !== 'undefined' && options.id > config.DEMO_ACCOUNTS.length) {
      throw Error(
        `Unable to find demo account for index '${options.id}'. Please make sure that you have specified enough demo accounts.`
      )
    }

    const providerUri = options?.provider || config.DEFAULT_URI
    const privateKey = usingSeed ? seed : stringToU8a(config.DEMO_ACCOUNTS[options.id])

    const provider = new Web3.providers.WebsocketProvider(providerUri, {
      reconnect: {
        auto: true,
        delay: 1000, // ms
        maxAttempts: 10,
      },
    })

    const web3 = new Web3(provider)

    const [network, publicKey] = await Promise.all([
      /* prettier-ignore */
      utils.getNetworkId(web3),
      utils.privKeyToPubKey(privateKey),
    ])

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
      network,
      hoprChannels,
      hoprToken,
      { debug: options?.debug || false },
      privateKey,
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
