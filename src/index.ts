import { randomBytes } from 'crypto'
import Web3 from 'web3'
import { LevelUp } from 'levelup'
import BN from 'bn.js'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import Channel, { events } from './channel'
import * as dbkeys from './dbKeys'
import * as types from './types'
import * as utils from './utils'
import * as constants from './constants'
import { u8aToHex, stringToU8a, u8aEquals } from './core/u8a'
import * as config from './config'
import { HoprChannels } from './tsc/web3/HoprChannels'
import { HoprToken } from './tsc/web3/HoprToken'
import HoprCoreConnector, {
  Utils as IUtils,
  Types as ITypes,
  Channel as IChannel,
  Constants as IConstants,
  DbKeys as IDbKeys
} from '@hoprnet/hopr-core-connector-interface'

export default class HoprEthereum implements HoprCoreConnector {
  private _accountSecretInitialized = false
  private _starting: Promise<void>
  private _started = false
  private _nonce?: number

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
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken
  ) {}

  readonly dbKeys = dbkeys as typeof IDbKeys
  readonly utils = utils as typeof IUtils
  readonly types = types as typeof ITypes
  readonly constants = constants as typeof IConstants
  readonly channel = Channel as typeof IChannel
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
    if (this._starting) {
      console.log('Connector is already starting..')
      return this._starting
    }
    if (this._started) {
      console.log('Connector has already started')
      return
    }

    this._starting = Promise.resolve()
      .then(async () => {
        // initialize stuff
        await Promise.all([
          // initialize account secret
          this.initializeAccountSecret()
          // ..
        ])

        // agnostic check if connector can start
        // @TODO: maybe remove this OR introduce timeout
        while (!this.canStart()) {
          await utils.wait(1 * 1e3)
        }

        this._started = true
      })
      .catch(console.error)
      .finally(() => {
        this._starting = undefined
      })

    return this._starting
  }

  async stop() {
    // @TODO: cancel starting
    if (this._starting) {
      console.log(`Cannot stop connector while it's starting`)
      return
    }

    events.clearAllEvents()
    this._started = false
  }

  get started() {
    return this._started
  }

  // check whether connector can start
  async canStart() {
    return this._accountSecretInitialized
  }

  async initOnchainValues(nonce?: number) {
    return this.setAccountSecret(nonce)
  }

  async initializeAccountSecret(): Promise<void> {
    console.log('Initializing account secret')
    const ok = await this.checkAccountSecret()

    if (!ok) {
      console.log('Setting account secret..')
      await this.setAccountSecret()
    }

    console.log('Account secret initialized!')
  }

  // return 'true' if account secret is setup correctly
  async checkAccountSecret(): Promise<boolean> {
    let offChainSecret: Uint8Array
    let onChainSecret: Uint8Array

    // retrieve offChain secret
    try {
      offChainSecret = await this.db.get(Buffer.from(dbkeys.OnChainSecret()))
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
          this.hoprChannels.methods.setHashedSecret(u8aToHex(offChainSecret)).send({
            from: this.account.toHex(),
            gas: 200e3
          })
        )
        hasOnChainSecret = true
      } else {
        throw Error(`Key is present on-chain but not in our database.`)
      }
    }

    return hasOffChainSecret && hasOnChainSecret
  }

  async setAccountSecret(nonce?: number): Promise<void> {
    let secret = new Uint8Array(randomBytes(32))

    const dbPromise = this.db.put(Buffer.from(this.dbKeys.OnChainSecret()), secret.slice())

    for (let i = 0; i < 500; i++) {
      secret = await this.utils.hash(secret)
    }

    await Promise.all([
      await utils.waitForConfirmation(
        this.hoprChannels.methods.setHashedSecret(u8aToHex(secret)).send({
          from: this.account.toHex(),
          nonce: nonce || (await this.nonce),
          gas: 200e3
        })
      ),
      dbPromise
    ])
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
    if (usingOptions && options.id > config.DEMO_ACCOUNTS.length) {
      throw Error(
        `Unable to find demo account for index '${options.id}'. Please make sure that you have specified enough demo accounts.`
      )
    }

    const provider = options?.provider || config.DEFAULT_URI
    const privateKey = usingSeed ? seed : stringToU8a(config.DEMO_ACCOUNTS[options.id])
    const publicKey = await utils.privKeyToPubKey(privateKey)
    const address = await utils.pubKeyToAccountId(publicKey)

    const web3 = new Web3(provider)
    const account = new types.AccountId(address)

    // add privkey to web3
    // TODO: check if this is good practise
    const web3Accounts = await web3.eth.getAccounts()
    if (!web3Accounts.includes(account.toHex())) {
      console.log('adding private key to web3js context')
      const acc = web3.eth.accounts.privateKeyToAccount(u8aToHex(privateKey))
      web3.eth.accounts.wallet.add(acc)
    }

    const network = await utils.getNetworkId(web3)

    if (typeof config.CHANNELS_ADDRESSES[network] === 'undefined') {
      throw Error(`channel contract address from network ${network} not found`)
    }
    if (typeof config.TOKEN_ADDRESSES[network] === 'undefined') {
      throw Error(`token contract address from network ${network} not found`)
    }

    const hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, config.CHANNELS_ADDRESSES[network])
    const hoprToken = new web3.eth.Contract(HoprTokenAbi as any, config.TOKEN_ADDRESSES[network])

    const hoprEthereum = new HoprEthereum(
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
      hoprChannels,
      hoprToken
    )

    return hoprEthereum
  }
}

export const Types = types
export const Utils = utils
