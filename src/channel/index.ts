import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import ChannelStatic from '@hoprnet/hopr-core-connector-interface/src/channel'
import { ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import { typedClass } from 'src/tsc/utils'
import { Uint8Array } from 'src/types/extended'
import { u8aToHex, u8aXOR } from 'src/core/u8a'
import { SignedChannel, Moment, Hash, AccountId, ChannelId, Balance, ChannelBalance, Ticket } from 'src/types'
import HoprEthereum from '..'
import { HASH_LENGTH } from 'src/constants'

declare namespace NChannel {
  interface Static<T extends HoprCoreConnectorInstance> extends ChannelStatic<T> {
    new (...props: any[]): Instance
  }
  interface Instance extends ChannelInstance {}
}

type IChannel = {
  deposit: string
  partyABalance: string
  closureTime: string
  stateCounter: string
}

const getChannel = ({ hoprEthereum, channelId }: { hoprEthereum: HoprEthereum; channelId: Hash }) => {
  return new Promise<IChannel>(async (resolve, reject) => {
    try {
      const channelIdHex = u8aToHex(channelId)
      const response = await hoprEthereum.hoprChannels.methods.channels(channelIdHex).call()
      return resolve(response)
    } catch (error) {
      return reject(error)
    }
  })
}

@typedClass<NChannel.Static<typeof HoprEthereum>>()
class Channel {
  private _signedChannel: SignedChannel
  private _settlementWindow?: Moment
  private _channelId?: Hash
  public counterpartyHex: string
  public partyAHex: string
  public partyBHex: string

  constructor(public hoprEthereum: HoprEthereum, public counterparty: AccountId, signedChannel: SignedChannel) {
    this.counterpartyHex = u8aToHex(counterparty)

    const parties = hoprEthereum.utils.getParties(hoprEthereum.account, counterparty)
    this.partyAHex = u8aToHex(parties[0])
    this.partyBHex = u8aToHex(parties[1])

    this._signedChannel = signedChannel
  }

  private get channel() {
    return new Promise<IChannel>(async (resolve, reject) => {
      try {
        const response = await getChannel({
          hoprEthereum: this.hoprEthereum,
          channelId: await this.channelId
        })
        return resolve(response)
      } catch (error) {
        return reject(error)
      }
    })
  }

  get offChainCounterparty(): Uint8Array {
    return this._signedChannel.signer
  }

  get channelId() {
    return new Promise<Hash>(async (resolve, reject) => {
      if (this._channelId != null) {
        return resolve(this._channelId)
      }

      try {
        const channelId = await this.hoprEthereum.utils.getId(this.hoprEthereum.account, this.counterparty)
        this._channelId = new ChannelId(channelId)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._channelId)
    })
  }

  get settlementWindow() {
    return new Promise<Moment>(async (resolve, reject) => {
      if (this._settlementWindow != null) {
        return resolve(this._settlementWindow)
      }

      try {
        const channel = await this.channel
        this._settlementWindow = new Moment(channel.closureTime)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._settlementWindow)
    })
  }

  get state() {
    return new Promise<Uint8Array>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new Uint8Array(Number(channel.stateCounter)))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance() {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new Balance(channel.deposit))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_a() {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new Balance(channel.partyABalance))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalance() {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const response = await this.hoprEthereum.hoprToken.methods.balanceOf(this.hoprEthereum.accountHex).call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalanceOfCounterparty() {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const response = await this.hoprEthereum.hoprToken.methods.balanceOf(this.counterpartyHex).call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  ticket = Ticket

  async initiateSettlement(): Promise<void> {
    try {
      await this.hoprEthereum.utils.waitForConfirmation(
        this.hoprEthereum.hoprChannels.methods.initiateChannelClosure(this.counterpartyHex).send({
          from: this.hoprEthereum.accountHex
        })
      )
    } catch (error) {
      throw error
    }
  }

  // TODO: this is broken
  async getPreviousChallenges(): Promise<Hash> {
    let pubKeys: Uint8Array[] = []
    const challenge = new Uint8Array(HASH_LENGTH).fill(0x00)

    return new Promise<Hash>(async (resolve, reject) => {
      this.hoprEthereum.db
        .createReadStream({
          gt: this.hoprEthereum.dbKeys.Challenge(await this.channelId, challenge),
          lt: this.hoprEthereum.dbKeys.Challenge(await this.channelId, challenge)
        })
        .on('error', reject)
        .on('data', ({ key, ownKeyHalf }) => {
          const [channelId, challenge] = this.hoprEthereum.dbKeys.ChallengeKeyParse(key)

          // BIG TODO !!
          // replace this by proper EC-arithmetic
          pubKeys.push(new Uint8Array(u8aXOR(false, challenge, ownKeyHalf.toU8a())))
        })
        .on('end', () => {
          if (pubKeys.length > 0) {
            return resolve(new Uint8Array(u8aXOR(false, ...pubKeys)))
          }

          resolve()
        })
    })
  }

  /**
   * Checks if there exists a payment channel with `counterparty`.
   * @param hoprEthereum the CoreConnector instance
   * @param counterparty secp256k1 public key of the counterparty
   */
  static async isOpen(hoprEthereum: HoprEthereum, counterparty: AccountId, channelId: Hash) {
    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      getChannel({
        hoprEthereum,
        channelId
      }).then(channel => {
        // TODO: double check
        return channel.stateCounter !== '0'
      }),
      hoprEthereum.db.get(hoprEthereum.dbKeys.Channel(counterparty)).then(
        () => true,
        (err: any) => {
          if (err.notFound) {
            return false
          } else {
            throw err
          }
        }
      )
    ])

    if (onChain != offChain) {
      if (!onChain && offChain) {
        throw Error(`Channel ${u8aToHex(channelId)} exists off-chain but not on-chain.`)
      } else {
        throw Error(`Channel ${u8aToHex(channelId)} exists on-chain but not off-chain.`)
      }
    }

    return onChain && offChain
  }

  /**
   * Checks whether the channel is open and opens that channel if necessary.
   * @param hoprEthereum the connector instance
   * @param offChainCounterparty public key used off-chain
   * @param getOnChainPublicKey yields the on-chain identity
   * @param channelBalance desired channel balance
   * @param sign signing provider
   */
  static async create(
    hoprEthereum: HoprEthereum,
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    let signedChannel: SignedChannel

    const counterparty = await getOnChainPublicKey(offChainCounterparty)
    const channelId = new Hash(await hoprEthereum.utils.getId(hoprEthereum.account, counterparty))

    if (await this.isOpen(hoprEthereum, counterparty, channelId)) {
      signedChannel = new SignedChannel(await hoprEthereum.db.get(hoprEthereum.dbKeys.Channel(counterparty)))
    } else if (sign != null && channelBalance != null) {
      const channelOpener = await ChannelOpener.create(hoprEthereum, counterparty, channelId)

      if (hoprEthereum.utils.isPartyA(hoprEthereum.account, counterparty)) {
        await channelOpener.increaseFunds(channelBalance.balance_a)
      } else {
        await channelOpener.increaseFunds(channelBalance.balance.sub(channelBalance.balance_a))
      }

      signedChannel = await sign(channelBalance)
      // await Promise.all([
      //   /* prettier-ignore */
      //   channelOpener.onceOpen(),
      //   channelOpener.setActive(signedChannel)
      // ])

      await hoprEthereum.db.put(hoprEthereum.dbKeys.Channel(counterparty), Buffer.from(signedChannel))
    } else {
      throw Error('Invalid input parameters.')
    }

    return new Channel(hoprEthereum, counterparty, signedChannel)
  }
}

export default Channel
