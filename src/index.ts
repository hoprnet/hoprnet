import { randomBytes } from 'crypto'
import Web3 from './web3'
import { LevelUp } from 'levelup'
import BN from 'bn.js'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprCoreConnector, {
  Utils as IUtils,
  Types as ITypes,
  Channel as IChannel,
  Constants as IConstants,
  DbKeys as IDbKeys
} from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import Channel, { events } from './channel'
import Ticket from './ticket'
import * as dbkeys from './dbKeys'
import * as types from './types'
import * as utils from './utils'
import * as constants from './constants'
import * as config from './config'
import { Networks } from './tsc/types'
import { HoprChannels } from './tsc/web3/HoprChannels'
import { HoprToken } from './tsc/web3/HoprToken'

export default class HoprEthereum implements HoprCoreConnector {
  private _status: 'uninitialized' | 'initialized' | 'started' | 'stopped' = 'uninitialized'
  private _initializing: Promise<void>
  private _starting: Promise<void>
  private _stopping: Promise<void>
  private _nonce?: number
  public signTransaction: ReturnType<typeof utils.TransactionSigner>

  constructor(
    public db: LevelUp,
    public self: {
      privateKey: Uint8Array
      publicKey: Uint8Array
      onChainKeyPair: {
        privateKey?: Uint8Array
        publicKey?: Uint8Array
      }
    },
    public account: types.AccountId,
    public web3: Web3,
    public network: Networks,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken
  ) {
    this.signTransaction = utils.TransactionSigner(web3, self.privateKey)
  }

  readonly dbKeys = dbkeys as typeof IDbKeys
  readonly utils = utils as typeof IUtils
  readonly types = types as typeof ITypes
  readonly constants = constants as typeof IConstants
  readonly channel = Channel as typeof IChannel
  readonly ticket = Ticket
  readonly CHAIN_NAME = 'HOPR on Ethereum'

  get nonce(): Promise<number> {
    return new Promise<number>(async (resolve, reject) => {
      if (typeof this._nonce !== 'undefined') {
        return this._nonce++
      }

      try {
        this._nonce = await this.web3.eth.getTransactionCount(this.account.toHex())
      } catch (error) {
        reject(error)
      }

      resolve(this._nonce++)
    })
  }

  get accountBalance() {
    return this.hoprToken.methods
      .balanceOf(u8aToHex(this.account))
      .call()
      .then(res => {
        return new BN(res)
      })
  }

  async start() {
    console.log('Starting connector..')

    if (typeof this._starting !== 'undefined') {
      console.log('Connector is already starting..')
      return this._starting
    } else if (this._status === 'started') {
      console.log('Connector has already started')
      return
    } else if (this._status === 'uninitialized' && typeof this._initializing === 'undefined') {
      console.log('Connector was asked to start but state was not asked to initialize, initializing..')
      this.initialize().catch(console.error)
    }

    this._starting = Promise.resolve()
      .then(async () => {
        // agnostic check if connector can start
        while (this._status !== 'initialized') {
          await utils.wait(1 * 1e3)
        }

        this.web3.events.on('reconnected', async () => {
          console.log('lost connection to web3, restarting..')

          await this.stop()
          await this.start()
        })

        this._status = 'started'
        console.log('Connector started')
      })
      .catch(err => {
        console.log('Connector failed to start')
        console.error(err)
      })
      .finally(() => {
        this._starting = undefined
      })

    return this._starting
  }

  async stop() {
    console.log('Stopping connector..')

    if (typeof this._stopping !== 'undefined') {
      console.log('Connector is already stopping..')
      return this._stopping
    } else if (this._status === 'stopped') {
      console.log('Connector has already stopped')
      return
    }

    this._stopping = Promise.resolve()
      .then(async () => {
        // connector is starting
        if (typeof this._starting !== 'undefined') {
          console.log("Connector will stop once it's started")
          // @TODO: cancel initializing & starting
          await this._starting
        }

        events.clearAllEvents()

        this._status = 'stopped'
        console.log('Connector stopped')
      })
      .catch(err => {
        console.log('Connector failed to stop')
        console.error(err)
      })
      .finally(() => {
        this._stopping = undefined
      })

    return this._stopping
  }

  get started() {
    return this._status === 'started'
  }

  async initOnchainValues(nonce?: number) {
    return this.setAccountSecret(nonce)
  }

  async initialize(): Promise<void> {
    console.log('Initializing connector..')

    if (typeof this._initializing !== 'undefined') {
      console.log('Connector is already initializing..')
      return this._initializing
    } else if (this._status === 'initialized') {
      console.log('Connector has already initialized')
      return
    } else if (this._status !== 'uninitialized') {
      throw Error(`invalid status '${this._status}', could not initialize`)
    }

    this._initializing = Promise.resolve()
      .then(async () => {
        // initialize stuff
        await Promise.all<boolean>([
          // initialize account secret
          this.initializeAccountSecret(),
          // confirm web3 is connected
          this.checkWeb3()
        ]).then(responses => {
          const allOk = responses.every(r => !!r)

          if (!allOk) {
            throw Error('could not initialize connector')
          }
        })

        this._status = 'initialized'
        console.log('Connector initialized')
      })
      .catch(err => {
        console.log('Connector failed to initialize')
        console.error(err)
      })
      .finally(() => {
        this._initializing = undefined
      })

    return this._initializing
  }

  async initializeAccountSecret(): Promise<boolean> {
    try {
      console.log('Initializing account secret')
      const ok = await this.checkAccountSecret()

      if (!ok) {
        console.log('Setting account secret..')
        await this.setAccountSecret()
      }

      console.log('Account secret initialized!')
      return true
    } catch (err) {
      console.log('error initializing account secret')
      console.error(err)
      return false
    }
  }

  // return 'true' if account secret is setup correctly
  async checkAccountSecret(): Promise<boolean> {
    let offChainSecret: Uint8Array
    let onChainSecret: Uint8Array

    // retrieve offChain secret
    try {
      offChainSecret = new Uint8Array(await this.db.get(Buffer.from(dbkeys.OnChainSecret())))
    } catch (err) {
      if (err.notFound != true) {
        throw err
      }
      offChainSecret = undefined
    }

    // retrieve onChain secret
    onChainSecret = await this.hoprChannels.methods
      .accounts(this.account.toHex())
      .call()
      .then(res => stringToU8a(res.hashedSecret))
      .then(secret => {
        if (u8aEquals(secret, new Uint8Array(types.Hash.SIZE).fill(0x00))) {
          return undefined
        }

        return secret
      })

    let hasOffChainSecret = typeof offChainSecret !== 'undefined'
    let hasOnChainSecret = typeof onChainSecret !== 'undefined'

    if (hasOffChainSecret !== hasOnChainSecret) {
      if (hasOffChainSecret) {
        console.log(`Key is present off-chain but not on-chain, submitting..`)
        await utils.waitForConfirmation(
          (
            await this.signTransaction(this.hoprChannels.methods.setHashedSecret(u8aToHex(offChainSecret)), {
              from: this.account.toHex(),
              to: this.hoprChannels.options.address
            })
          ).send()
        )
        hasOnChainSecret = true
      } else {
        throw Error(`Key is present on-chain but not in our database.`)
      }
    }

    return hasOffChainSecret && hasOnChainSecret
  }

  // generate and set account secret
  async setAccountSecret(nonce?: number): Promise<void> {
    let secret = new Uint8Array(randomBytes(32))
    const dbPromise = this.db.put(Buffer.from(this.dbKeys.OnChainSecret()), Buffer.from(secret.slice()))

    for (let i = 0; i < 500; i++) {
      secret = await this.utils.hash(secret)
    }

    await Promise.all([
      await utils.waitForConfirmation(
        (
          await this.signTransaction(this.hoprChannels.methods.setHashedSecret(u8aToHex(secret)), {
            from: this.account.toHex(),
            to: this.hoprChannels.options.address,
            nonce: nonce || (await this.nonce)
          })
        ).send()
      ),
      dbPromise
    ])
  }

  async checkWeb3(): Promise<boolean> {
    try {
      const isListening = await this.web3.eth.net.isListening()
      if (!isListening) throw Error('web3 is not connected')

      return true
    } catch (err) {
      console.log('error checking web3')
      console.error(err)
      return false
    }
  }

  static readonly constants = constants as typeof IConstants

  static async create(
    db: LevelUp,
    seed?: Uint8Array,
    options?: { id?: number; provider?: string }
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

    const provider = options?.provider || config.DEFAULT_URI
    const privateKey = usingSeed ? seed : stringToU8a(config.DEMO_ACCOUNTS[options.id])
    const publicKey = await utils.privKeyToPubKey(privateKey)
    const address = await utils.pubKeyToAccountId(publicKey)

    const web3 = new Web3(provider)
    // @TODO: stop using this
    await web3.isConnected()

    const account = new types.AccountId(address)
    console.log(`using ethereum address ${account.toHex()}`)
    const network = await utils.getNetworkId(web3)

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
      {
        privateKey,
        publicKey,
        onChainKeyPair: {
          privateKey,
          publicKey
        }
      },
      account,
      web3,
      network,
      hoprChannels,
      hoprToken
    )

    // begin initializing
    coreConnector.initialize().catch(console.error)
    coreConnector.start()

    return coreConnector
  }
}

export const Types = types
export const Utils = utils
