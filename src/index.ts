import assert from 'assert'
import { randomBytes } from 'crypto'
import Web3 from 'web3'
import { LevelUp } from 'levelup'
import BN from 'bn.js'
import { HoprCoreConnectorInstance, Types as ITypes } from '@hoprnet/hopr-core-connector-interface'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import Channel from './channel'
import DbKeysClass from './dbKeys'
import * as Types from './types'
import * as Utils from './utils'
import * as constants from './constants'
import { u8aToHex } from './core/u8a'
import { HoprChannels } from './tsc/web3/HoprChannels'
import { HoprToken } from './tsc/web3/HoprToken'
import { DEFAULT_URI, DEFAULT_HOPR_CHANNELS_ADDRESS, DEFAULT_HOPR_TOKEN_ADDRESS } from './config'

const DbKeys = new DbKeysClass()

type HoprKeyPair = {
  privateKey: Uint8Array
  publicKey: Uint8Array
}

export default class HoprEthereumClass implements HoprCoreConnectorInstance {
  private _started: boolean = false
  private _nonce?: number

  constructor(
    public db: LevelUp,
    public self: {
      publicKey: Uint8Array
      privateKey: Uint8Array
    },
    public account: ITypes.AccountId,
    public web3: Web3,
    public hoprChannels: HoprChannels,
    public hoprToken: HoprToken
  ) {}

  readonly dbKeys = DbKeys
  readonly utils = Utils
  readonly types = Types
  readonly channel = Channel
  readonly constants = constants
  readonly CHAIN_NAME = 'HOPR on Ethereum'

  get nonce() {
    return new Promise<number>(async (resolve, reject) => {
      if (typeof this._nonce !== 'undefined') {
        return this._nonce++
      }

      try {
        this._nonce = await this.web3.eth.getTransactionCount(u8aToHex(this.account))
      } catch (error) {
        reject(error)
      }

      resolve(this._nonce++)
    })
  }

  async start() {
    this._started = true
  }

  async stop() {
    this._started = false
  }

  get started() {
    return this._started
  }

  async initOnchainValues(nonce?: number) {
    assert(this.started, 'Module is not yet fully initialised.')

    let secret = new Uint8Array(randomBytes(32))

    for (let i = 0; i < 1000; i++) {
      secret = await this.utils.hash(secret)
    }

    await this.utils.waitForConfirmation(
      this.hoprChannels.methods.setHashedSecret(u8aToHex(secret)).send({
        from: u8aToHex(this.account),
        nonce: await this.nonce
      })
    )

    await this.db.put(this.dbKeys.OnChainSecret(), secret)
  }

  get accountBalance() {
    return this.hoprToken.methods
      .balanceOf(u8aToHex(this.account))
      .call()
      .then(res => {
        return new BN(res)
      })
  }

  static async create(db: LevelUp, keyPair: HoprKeyPair): Promise<HoprEthereumClass> {
    const web3 = new Web3(DEFAULT_URI)
    const account = await Utils.hash(keyPair.publicKey)
    const hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, DEFAULT_HOPR_CHANNELS_ADDRESS)
    const hoprToken = new web3.eth.Contract(HoprTokenAbi as any, DEFAULT_HOPR_TOKEN_ADDRESS)

    return new HoprEthereumClass(db, keyPair, account, web3, hoprChannels, hoprToken)
  }
}

export type { HoprEthereumClass }
