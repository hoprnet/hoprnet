import { randomBytes } from 'crypto'
import Web3 from 'web3'
import { LevelUp } from 'levelup'
import BN from 'bn.js'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import Channel from './channel'
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
  private _started: boolean = false
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

  get nonce() {
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

  async start() {
    this._started = true
  }

  // TODO: unsubscribe event listeners
  async stop() {
    this._started = false
  }

  get started() {
    return this._started
  }

  async initOnchainValues(nonce?: number) {
    let secret = new Uint8Array(randomBytes(32))

    const dbPromise = this.db.put(Buffer.from(this.dbKeys.OnChainSecret()), secret.slice())

    for (let i = 0; i < 500; i++) {
      secret = await this.utils.hash(secret)
    }

    await Promise.all([
      await utils.waitForConfirmation(
        this.hoprChannels.methods.setHashedSecret(u8aToHex(secret)).send({
          from: this.account.toHex(),
          nonce: nonce || (await this.nonce)
        })
      ),
      dbPromise
    ])
  }

  get accountBalance() {
    return this.hoprToken.methods
      .balanceOf(u8aToHex(this.account))
      .call()
      .then(res => {
        return new BN(res)
      })
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
    const hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, config.DEFAULT_HOPR_CHANNELS_ADDRESS)
    const hoprToken = new web3.eth.Contract(HoprTokenAbi as any, config.DEFAULT_HOPR_TOKEN_ADDRESS)

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

    // if (!(await checkOnChainValues(hoprChannels, db, account))) {
    //   await hoprEthereum.initOnchainValues()
    // }

    return hoprEthereum
  }
}

// TODO: test
async function checkOnChainValues(hoprChannels: HoprChannels, db: LevelUp, account: types.AccountId) {
  let offChain: boolean
  let secret: Uint8Array = new Uint8Array()

  try {
    secret = await db.get(Buffer.from(dbkeys.OnChainSecret()))
    offChain = true
  } catch (err) {
    if (err.notFound != true) {
      throw err
    }
    offChain = false
  }

  const onChainSecret = await hoprChannels.methods
    .accounts(account.toHex())
    .call()
    .then(res => res.hashedSecret)
    .then(hashedSecret => stringToU8a(hashedSecret))

  const onChain = !u8aEquals(onChainSecret, new Uint8Array(types.Hash.SIZE).fill(0x00))

  // if (offChain != onChain) {
  //   if (offChain) {
  //     await hoprChannels.methods.setHashedSecret(u8aToHex(secret)).send({
  //       from: account.toHex()
  //     })
  //   } else {
  //     throw Error(`Key is present on-chain but not in our database.`)
  //   }
  // }

  return offChain && onChain
}

export const Types = types
export const Utils = utils
