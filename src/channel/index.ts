import type { Channel as IChannel, Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, u8aXOR, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import {
  SignedChannel,
  Moment,
  Hash,
  AccountId,
  ChannelId,
  Balance,
  ChannelBalance,
  Ticket,
  State,
  Public,
  TicketEpoch,
} from '../types'
import { ChannelStatus } from '../types/channel'
import { HASH_LENGTH, ERRORS } from '../constants'
import { waitForConfirmation, waitFor, hash, getId, stateCountToStatus, cleanupPromiEvent } from '../utils'
import type HoprEthereum from '..'

async function getChannel(
  coreConnector: HoprEthereum,
  channelId: Hash
): Promise<{
  deposit: string
  partyABalance: string
  closureTime: string
  stateCounter: string
}> {
  return coreConnector.hoprChannels.methods.channels(channelId.toHex()).call()
}

const onceOpen = async (coreConnector: HoprEthereum, self: AccountId, counterparty: AccountId) => {
  const channelId = await getId(self, counterparty)

  return cleanupPromiEvent(
    coreConnector.hoprChannels.events.OpenedChannel({
      filter: {
        opener: [self.toHex(), counterparty.toHex()],
        counterParty: [self.toHex(), counterparty.toHex()],
      },
    }),
    (event) => {
      return new Promise<{
        opener: string
        counterParty: string
      }>((resolve, reject) => {
        event
          .on('data', async (data) => {
            const { opener, counterParty } = data.returnValues
            const _channelId = await coreConnector.utils.getId(
              new AccountId(stringToU8a(opener)),
              new AccountId(stringToU8a(counterParty))
            )

            if (!u8aEquals(_channelId, channelId)) {
              return
            }

            return resolve(data.returnValues)
          })
          .on('error', reject)
      })
    }
  )
}

const onceClosed = async (coreConnector: HoprEthereum, self: AccountId, counterparty: AccountId) => {
  const channelId = await getId(self, counterparty)

  return cleanupPromiEvent(
    coreConnector.hoprChannels.events.ClosedChannel({
      filter: {
        closer: [self.toHex(), counterparty.toHex()],
        counterParty: [self.toHex(), counterparty.toHex()],
      },
    }),
    (event) => {
      return new Promise<{
        closer: string
        counterParty: string
      }>((resolve, reject) => {
        event
          .on('data', async (data) => {
            const { closer, counterParty } = data.returnValues
            const _channelId = await coreConnector.utils.getId(
              new AccountId(stringToU8a(closer)),
              new AccountId(stringToU8a(counterParty))
            )

            if (!u8aEquals(_channelId, channelId)) {
              return
            }

            resolve(data.returnValues)
          })
          .on('error', reject)
      })
    }
  )
}

const onOpen = async (coreConnector: HoprEthereum, counterparty: Uint8Array, signedChannel: SignedChannel) => {
  return coreConnector.db.put(Buffer.from(coreConnector.dbKeys.Channel(counterparty)), Buffer.from(signedChannel))
}

const onClose = async (coreConnector: HoprEthereum, counterparty: Uint8Array) => {
  return coreConnector.db.del(Buffer.from(coreConnector.dbKeys.Channel(counterparty)))
}

class Channel implements IChannel<HoprEthereum> {
  private _signedChannel: SignedChannel
  private _settlementWindow?: Moment
  private _channelId?: Hash

  ticket = Ticket as typeof Types.Ticket

  constructor(public coreConnector: HoprEthereum, public counterparty: Uint8Array, signedChannel: SignedChannel) {
    this._signedChannel = signedChannel

    // check if channel still exists
    this.status.then((status) => {
      if (status === ChannelStatus.UNINITIALISED) {
        this.coreConnector.log.log('found channel off-chain but its closed on-chain')
        this.onClose()
      }
    })

    // if channel is closed
    this.onceClosed().then(async () => {
      return this.onClose()
    })
  }

  // private async onceOpen() {
  //   return onceOpen(
  //     this.coreConnector,
  //     this.coreConnector.account,
  //     await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
  //   )
  // }

  private async onceClosed() {
    return onceClosed(
      this.coreConnector,
      this.coreConnector.account,
      await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
    )
  }

  // private async onOpen(): Promise<void> {
  //   return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
  // }

  private async onClose(): Promise<void> {
    return onClose(this.coreConnector, this.counterparty)
  }

  private get channel(): ReturnType<typeof getChannel> {
    return new Promise(async (resolve, reject) => {
      try {
        const response = await getChannel(this.coreConnector, await this.channelId)
        return resolve(response)
      } catch (error) {
        return reject(error)
      }
    })
  }

  private get status(): Promise<ChannelStatus> {
    return new Promise<ChannelStatus>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        const status = Number(channel.stateCounter) % 10

        if (status >= Object.keys(ChannelStatus).length) {
          throw Error("status like this doesn't exist")
        }

        return resolve(status)
      } catch (error) {
        return reject(error)
      }
    })
  }

  get offChainCounterparty(): Promise<Uint8Array> {
    return Promise.resolve(this.counterparty)
  }

  get channelId(): Promise<Hash> {
    if (this._channelId != null) {
      return Promise.resolve<Hash>(this._channelId)
    }

    return new Promise<Hash>(async (resolve, reject) => {
      try {
        const channelId = await this.coreConnector.utils.getId(
          this.coreConnector.account,
          await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
        )
        this._channelId = new ChannelId(channelId)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._channelId)
    })
  }

  get settlementWindow(): Promise<Moment> {
    if (this._settlementWindow != null) {
      return Promise.resolve(this._settlementWindow)
    }

    return new Promise<Moment>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        this._settlementWindow = new Moment(channel.closureTime)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._settlementWindow)
    })
  }

  get state(): Promise<State> {
    return new Promise<State>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        const status = stateCountToStatus(Number(channel.stateCounter))

        return resolve(
          new State(undefined, {
            // @TODO: implement this once on-chain channel secrets are added
            secret: new Hash(new Uint8Array(Hash.SIZE).fill(0x0)),
            // not needed
            pubkey: new Public(new Uint8Array(Public.SIZE).fill(0x0)),
            epoch: new TicketEpoch(status),
          })
        )
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new Balance(channel.deposit))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_a(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        return resolve(new Balance(channel.partyABalance))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalance(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const response = await this.coreConnector.hoprToken.methods
          .balanceOf(u8aToHex(this.coreConnector.account))
          .call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalanceOfCounterparty(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const response = await this.coreConnector.hoprToken.methods
          .balanceOf(u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
          .call()
        return resolve(new Balance(response))
      } catch (error) {
        return reject(error)
      }
    })
  }

  async initiateSettlement(): Promise<void> {
    // @TODO check out whether we can cache this.channel is some way
    let channel = await this.channel
    const status = await this.status

    try {
      if (!(status === ChannelStatus.OPEN || status === ChannelStatus.PENDING)) {
        throw Error("channel must be 'OPEN' or 'PENDING'")
      }

      if (status === ChannelStatus.OPEN) {
        await waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              this.coreConnector.hoprChannels.methods.initiateChannelClosure(
                u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
              ),
              {
                from: this.coreConnector.account.toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.nonce,
              }
            )
          ).send()
        )

        channel = await getChannel(this.coreConnector, await this.channelId)

        await waitFor({
          web3: this.coreConnector.web3,
          network: this.coreConnector.network,
          getCurrentBlock: async () => {
            return this.coreConnector.web3.eth.getBlockNumber().then((blockNumber) => {
              return this.coreConnector.web3.eth.getBlock(blockNumber)
            })
          },
          timestamp: Number(channel.closureTime) * 1e3,
        })

        await waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              this.coreConnector.hoprChannels.methods.claimChannelClosure(
                u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
              ),
              {
                from: this.coreConnector.account.toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.nonce,
              }
            )
          ).send()
        )
      } else {
        await this.onceClosed()
      }

      await this.onClose()
    } catch (error) {
      throw error
    }
  }

  async getPreviousChallenges(): Promise<Hash> {
    let pubKeys: Uint8Array[] = []

    return new Promise<Hash>(async (resolve, reject) => {
      this.coreConnector.db
        .createReadStream({
          gte: Buffer.from(
            this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(HASH_LENGTH).fill(0x00))
          ),
          lte: Buffer.from(
            this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(HASH_LENGTH).fill(0xff))
          ),
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, ownKeyHalf }: { key: Buffer; ownKeyHalf: Buffer }) => {
          const challenge = this.coreConnector.dbKeys.ChallengeKeyParse(key)[1]

          // @TODO: replace this by proper EC-arithmetic once it's implemented in `hopr-core`
          pubKeys.push(new Uint8Array(u8aXOR(false, challenge, new Uint8Array(ownKeyHalf))))
        })
        .on('end', () => {
          if (pubKeys.length > 0) {
            return resolve(new Hash(u8aXOR(false, ...pubKeys)))
          }

          resolve()
        })
    })
  }

  async testAndSetNonce(signature: Uint8Array): Promise<void> {
    const channelId = await this.channelId
    const nonce = await hash(signature)

    const key = new Hash(this.coreConnector.dbKeys.Nonce(channelId, nonce)).toHex()

    try {
      await this.coreConnector.db.get(key)
    } catch (err) {
      if (err.notFound == null || err.notFound != true) {
        throw err
      }
      await this.coreConnector.db.put(key, new Uint8Array())
      return
    }

    throw Error('Nonces must not be used twice.')
  }

  static async isOpen(coreConnector: HoprEthereum, counterpartyPubKey: Uint8Array) {
    const counterparty = await coreConnector.utils.pubKeyToAccountId(counterpartyPubKey)
    const channelId = await coreConnector.utils.getId(coreConnector.account, counterparty).then((res) => new Hash(res))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      getChannel(coreConnector, channelId).then((channel) => {
        const state = Number(channel.stateCounter) % 10
        return state === ChannelStatus.OPEN || state === ChannelStatus.PENDING
      }),
      coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Channel(counterpartyPubKey))).then(
        () => true,
        (err) => {
          if (err.notFound) {
            return false
          } else {
            throw err
          }
        }
      ),
    ])

    if (onChain != offChain) {
      if (!onChain && offChain) {
        coreConnector.log(`Channel ${u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`)
        await onClose(coreConnector, counterpartyPubKey)
      } else {
        throw Error(`Channel ${u8aToHex(channelId)} exists on-chain but not off-chain.`)
      }
    }

    return onChain && offChain
  }

  static async increaseFunds(coreConnector: HoprEthereum, counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      if ((await coreConnector.accountBalance).lt(amount)) {
        throw Error(ERRORS.OOF_HOPR)
      }

      await waitForConfirmation(
        (
          await coreConnector.signTransaction(
            coreConnector.hoprToken.methods.send(
              coreConnector.hoprChannels.options.address,
              amount.toString(),
              coreConnector.web3.eth.abi.encodeParameters(
                ['address', 'address'],
                [coreConnector.account.toHex(), counterparty.toHex()]
              )
            ),
            {
              from: coreConnector.account.toHex(),
              to: coreConnector.hoprToken.options.address,
              nonce: await coreConnector.nonce,
            }
          )
        ).send()
      )
    } catch (error) {
      throw error
    }
  }

  static async create(
    coreConnector: HoprEthereum,
    counterpartyPubKey: Uint8Array,
    _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    const counterparty = new AccountId(await coreConnector.utils.pubKeyToAccountId(counterpartyPubKey))
    let channel: Channel
    let signedChannel: SignedChannel

    if (await this.isOpen(coreConnector, counterpartyPubKey)) {
      const record = await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Channel(counterpartyPubKey)))
      signedChannel = new SignedChannel({
        bytes: record.buffer,
        offset: record.byteOffset,
      })
      channel = new Channel(coreConnector, counterpartyPubKey, signedChannel)
    } else if (sign != null && channelBalance != null) {
      let amount: Balance
      if (coreConnector.utils.isPartyA(coreConnector.account, counterparty)) {
        amount = channelBalance.balance_a
      } else {
        amount = new Balance(channelBalance.balance.sub(channelBalance.balance_a))
      }

      await Channel.increaseFunds(coreConnector, counterparty, amount)

      signedChannel = await sign(channelBalance)
      channel = new Channel(coreConnector, counterpartyPubKey, signedChannel)

      await waitForConfirmation(
        (
          await coreConnector.signTransaction(coreConnector.hoprChannels.methods.openChannel(counterparty.toHex()), {
            from: coreConnector.account.toHex(),
            to: coreConnector.hoprChannels.options.address,
            nonce: await coreConnector.nonce,
          })
        ).send()
      )

      await coreConnector.db.put(
        Buffer.from(coreConnector.dbKeys.Channel(counterpartyPubKey)),
        Buffer.from(signedChannel)
      )
    } else {
      throw Error('Invalid input parameters.')
    }

    return channel
  }

  static getAll<T, R>(
    coreConnector: HoprEthereum,
    onData: (channel: Channel) => Promise<T>,
    onEnd: (promises: Promise<T>[]) => R
  ): Promise<R> {
    const promises: Promise<T>[] = []
    return new Promise<R>((resolve, reject) => {
      coreConnector.db
        .createReadStream({
          gte: Buffer.from(coreConnector.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00))),
          lte: Buffer.from(coreConnector.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff))),
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = new SignedChannel({
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          promises.push(onData(new Channel(coreConnector, coreConnector.dbKeys.ChannelKeyParse(key), signedChannel)))
        })
        .on('end', () => resolve(onEnd(promises)))
    })
  }

  static async closeChannels(coreConnector: HoprEthereum): Promise<Balance> {
    const result = new BN(0)

    return Channel.getAll(
      coreConnector,
      (channel: Channel) =>
        channel.initiateSettlement().then(() => {
          // @TODO: add balance
          result.iaddn(0)
        }),
      async (promises: Promise<void>[]) => {
        await Promise.all(promises)

        return new Balance(result)
      }
    )
  }

  static handleOpeningRequest(
    coreConnector: HoprEthereum
  ): (source: AsyncIterable<Uint8Array>) => AsyncIterator<Uint8Array> {
    return (source) => {
      return (async function* () {
        for await (const _msg of source) {
          const msg = _msg.slice()
          const signedChannel = new SignedChannel({
            bytes: msg.buffer,
            offset: msg.byteOffset,
          })

          const counterpartyPubKey = await signedChannel.signer
          const counterparty = new AccountId(await coreConnector.utils.pubKeyToAccountId(counterpartyPubKey))
          const channelBalance = signedChannel.channel.balance

          if (coreConnector.utils.isPartyA(coreConnector.account, counterparty)) {
            if (channelBalance.balance.sub(channelBalance.balance_a).gtn(0)) {
              await Channel.increaseFunds(
                coreConnector,
                counterparty,
                new Balance(channelBalance.balance.sub(channelBalance.balance_a))
              )
            }
          } else {
            if (channelBalance.balance_a.gtn(0)) {
              await Channel.increaseFunds(coreConnector, counterparty, channelBalance.balance_a)
            }
          }

          // listen for opening event and update DB
          onceOpen(coreConnector, coreConnector.account, counterparty).then(() =>
            onOpen(coreConnector, counterpartyPubKey, signedChannel)
          )

          yield signedChannel.toU8a()
        }
      })()
    }
  }
}

export default Channel
