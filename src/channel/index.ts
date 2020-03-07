import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import ChannelStatic from '@hoprnet/hopr-core-connector-interface/src/channel'
import { ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import { typedClass } from 'src/tsc/utils'
import { Uint8ArrayE } from 'src/types/extended'
import { u8aToHex, u8aXOR, toU8a, stringToU8a, u8aEquals } from 'src/core/u8a'
import { SignedChannel, Moment, Hash, AccountId, ChannelId, Balance, ChannelBalance, Ticket, State } from 'src/types'
import { HASH_LENGTH } from 'src/constants'
import { waitForConfirmation } from 'src/utils'
import BN from 'bn.js'
import { HoprChannels as IHoprChannels } from 'src/tsc/web3/HoprChannels'
import HoprEthereum from '..'

declare namespace NChannel {
  interface Static extends ChannelStatic<HoprCoreConnectorInstance> {
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

const onceOpen = (hoprEthereum: HoprEthereum, channelId: Hash): Promise<void> => {
  let event: ReturnType<IHoprChannels['events']['OpenedChannel']>

  return new Promise<void>((resolve, reject) => {
    // TODO: better to filter
    event = hoprEthereum.hoprChannels.events
      .OpenedChannel()
      .on('data', async data => {
        const { opener, counterParty } = data.returnValues
        const _channelId = await hoprEthereum.utils.getId(stringToU8a(opener), stringToU8a(counterParty))

        if (!u8aEquals(_channelId, channelId)) {
          return
        }

        resolve()
      })
      .on('error', error => {
        reject(error)
      })
  }).finally(() => {
    event.removeAllListeners()
  })
}

const onceClosed = (hoprEthereum: HoprEthereum, channelId: Hash): Promise<void> => {
  let event: ReturnType<IHoprChannels['events']['ClosedChannel']>

  return new Promise<void>((resolve, reject) => {
    // TODO: better to filter
    event = hoprEthereum.hoprChannels.events
      .ClosedChannel()
      .on('data', async data => {
        const { closer, counterParty } = data.returnValues
        const _channelId = await hoprEthereum.utils.getId(stringToU8a(closer), stringToU8a(counterParty))

        if (!u8aEquals(_channelId, channelId)) {
          return
        }

        resolve()
      })
      .on('error', error => {
        reject(error)
      })
  }).finally(() => {
    event.removeAllListeners()
  })
}

@typedClass<NChannel.Static>()
class Channel {
  private _signedChannel: SignedChannel
  private _settlementWindow?: Moment
  private _channelId?: Hash

  constructor(public hoprEthereum: HoprEthereum, public counterparty: AccountId, signedChannel: SignedChannel) {
    this._signedChannel = signedChannel
  }

  private async onceOpen(): Promise<void> {
    return onceOpen(this.hoprEthereum, await this.channelId)
  }

  private async onceClosed(): Promise<void> {
    return onceClosed(this.hoprEthereum, await this.channelId)
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

  ticket = Ticket

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
    return new Promise<State>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new State(Number(channel.stateCounter)))
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
        const response = await this.hoprEthereum.hoprToken.methods.balanceOf(u8aToHex(this.hoprEthereum.account)).call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalanceOfCounterparty() {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const response = await this.hoprEthereum.hoprToken.methods.balanceOf(u8aToHex(this.counterparty)).call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  async initiateSettlement(): Promise<void> {
    try {
      await Promise.all([
        this.onceClosed(),
        this.hoprEthereum.hoprChannels.methods.initiateChannelClosure(u8aToHex(this.counterparty)).send({
          from: u8aToHex(this.hoprEthereum.account)
        })
      ])
    } catch (error) {
      throw error
    }
  }

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
            return resolve(new Hash(u8aXOR(false, ...pubKeys)))
          }

          resolve()
        })
    })
  }

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
        err => {
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

  static async increaseFunds(hoprEthereum: HoprEthereum, counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      if ((await hoprEthereum.accountBalance).lt(amount)) {
        throw Error('Insufficient funds.')
      }

      await waitForConfirmation(
        hoprEthereum.hoprChannels.methods
          .fundChannel(u8aToHex(hoprEthereum.account), u8aToHex(counterparty), amount.toString())
          .send({
            from: u8aToHex(hoprEthereum.account)
          })
      )
    } catch (error) {
      throw error
    }
  }

  static async create(
    hoprEthereum: HoprEthereum,
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    let signedChannel: SignedChannel

    const counterparty = await getOnChainPublicKey(offChainCounterparty).then(v => new Uint8ArrayE(v))
    const channelId = new Hash(await hoprEthereum.utils.getId(hoprEthereum.account, counterparty))
    let channel: Channel

    if (await this.isOpen(hoprEthereum, counterparty, channelId)) {
      signedChannel = new SignedChannel(await hoprEthereum.db.get(hoprEthereum.dbKeys.Channel(counterparty)))
      channel = new Channel(hoprEthereum, counterparty, signedChannel)
    } else if (sign != null && channelBalance != null) {
      if (hoprEthereum.utils.isPartyA(hoprEthereum.account, counterparty)) {
        await Channel.increaseFunds(hoprEthereum, counterparty, channelBalance.balance_a)
      } else {
        await Channel.increaseFunds(
          hoprEthereum,
          counterparty,
          new Balance(channelBalance.balance.sub(channelBalance.balance_a))
        )
      }

      signedChannel = await sign(channelBalance)
      channel = new Channel(hoprEthereum, counterparty, signedChannel)

      await channel.onceOpen()

      await hoprEthereum.db.put(hoprEthereum.dbKeys.Channel(counterparty), Buffer.from(signedChannel))
    } else {
      throw Error('Invalid input parameters.')
    }

    return channel
  }

  static getAll<T, R>(
    hoprEthereum: HoprEthereum,
    onData: (channel: Channel) => Promise<T>,
    onEnd: (promises: Promise<T>[]) => R
  ): Promise<R> {
    const promises: Promise<T>[] = []
    return new Promise<R>((resolve, reject) => {
      hoprEthereum.db
        .createReadStream({
          gt: hoprEthereum.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00)),
          lt: hoprEthereum.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff))
        })
        .on('error', err => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = new SignedChannel({
            bytes: value.buffer,
            offset: value.byteOffset
          })

          promises.push(
            onData(new Channel(hoprEthereum, new AccountId(hoprEthereum.dbKeys.ChannelKeyParse(key)), signedChannel))
          )
        })
        .on('end', () => resolve(onEnd(promises)))
    })
  }

  static async closeChannels(hoprEthereum: HoprEthereum): Promise<Balance> {
    const result = new BN(0)

    return Channel.getAll(
      hoprEthereum,
      (channel: Channel) =>
        channel.initiateSettlement().then(() => {
          // TODO: add balance
          result.iaddn(0)
        }),
      async (promises: Promise<void>[]) => {
        await Promise.all(promises)

        return new Balance(result)
      }
    )
  }

  async testAndSetNonce(signature: Uint8Array): Promise<void> {
    const key = this.hoprEthereum.dbKeys.Nonce(await this.channelId, toU8a(await this.hoprEthereum.nonce))

    try {
      await this.hoprEthereum.db.get(u8aToHex(key))
    } catch (err) {
      if (err.notFound == null || err.notFound != true) {
        throw err
      }
      return
    }

    throw Error('Nonces must not be used twice.')
  }

  static handleOpeningRequest(
    hoprEthereum: HoprEthereum
  ): (source: AsyncIterable<Uint8Array>) => AsyncIterator<Uint8Array> {
    return source => {
      return (async function*(msgs) {
        for await (const msg of msgs) {
          const signedChannel = new SignedChannel({
            bytes: msg.buffer,
            offset: msg.byteOffset
          })

          const counterparty = signedChannel.signer
          const channelBalance = signedChannel.channel.balance
          const channelId = await hoprEthereum.utils.getId(hoprEthereum.account, counterparty)

          await onceOpen(hoprEthereum, new Hash(channelId))

          if (hoprEthereum.utils.isPartyA(hoprEthereum.account, counterparty)) {
            await Channel.increaseFunds(hoprEthereum, counterparty, channelBalance.balance_a)
          } else {
            await Channel.increaseFunds(
              hoprEthereum,
              counterparty,
              new Balance(channelBalance.balance.sub(channelBalance.balance_a))
            )

            await hoprEthereum.db.put(u8aToHex(hoprEthereum.dbKeys.Channel(counterparty)), Buffer.from(signedChannel))
          }

          return signedChannel
        }
      })(source)
    }
  }
}

export default Channel
