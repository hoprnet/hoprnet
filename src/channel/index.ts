import type { Channel as IChannel, Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex, u8aXOR, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import {
  AccountId,
  Balance,
  ChannelBalance,
  ChannelId,
  Hash,
  Moment,
  Public,
  SignedChannel,
  SignedTicket,
  State,
  Ticket,
  TicketEpoch,
} from '../types'
import { ChannelStatus } from '../types/channel'
import { HASH_LENGTH, ERRORS } from '../constants'
import { waitForConfirmation, waitFor, hash, getId, stateCountToStatus, cleanupPromiEvent } from '../utils'
import type HoprEthereum from '..'

import { Uint8ArrayE } from '../types/extended'
import { randomBytes } from 'crypto'

const WIN_PROB = new BN(1)

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

class Channel implements IChannel {
  private _signedChannel: SignedChannel
  private _settlementWindow?: Moment
  private _channelId?: Hash

  public ticket: typeof Types.Ticket

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

    this.ticket = {
      create: Ticket.create.bind(this),
      SIZE: Ticket.SIZE,
      submit: Ticket.submit.bind(this),
      verify: Ticket.verify.bind(this),
    }
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
        this._channelId = new ChannelId(
          await this.coreConnector.utils.getId(
            this.coreConnector.account,
            await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
          )
        )
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
        this._settlementWindow = new Moment((await this.channel).closureTime)
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
        return resolve(new Balance((await this.channel).deposit))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_a(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(new Balance((await this.channel).partyABalance))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalance(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(
          new Balance(await this.coreConnector.hoprToken.methods.balanceOf(u8aToHex(this.coreConnector.account)).call())
        )
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalanceOfCounterparty(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(
          new Balance(
            await this.coreConnector.hoprToken.methods
              .balanceOf(u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
              .call()
          )
        )
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

  static async isOpen(this: HoprEthereum, counterpartyPubKey: Uint8Array) {
    const counterparty = await this.utils.pubKeyToAccountId(counterpartyPubKey)
    const channelId = await this.utils.getId(this.account, counterparty).then((res) => new Hash(res))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      getChannel(this, channelId).then((channel) => {
        const state = Number(channel.stateCounter) % 10
        return state === ChannelStatus.OPEN || state === ChannelStatus.PENDING
      }),
      this.db.get(Buffer.from(this.dbKeys.Channel(counterpartyPubKey))).then(
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
        this.log(`Channel ${u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`)
        await onClose(this, counterpartyPubKey)
      } else {
        throw Error(`Channel ${u8aToHex(channelId)} exists on-chain but not off-chain.`)
      }
    }

    return onChain && offChain
  }

  static async increaseFunds(this: HoprEthereum, counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      if ((await this.accountBalance).lt(amount)) {
        throw Error(ERRORS.OOF_HOPR)
      }

      await waitForConfirmation(
        (
          await this.signTransaction(
            this.hoprToken.methods.send(
              this.hoprChannels.options.address,
              amount.toString(),
              this.web3.eth.abi.encodeParameters(['address', 'address'], [this.account.toHex(), counterparty.toHex()])
            ),
            {
              from: this.account.toHex(),
              to: this.hoprToken.options.address,
              nonce: await this.nonce,
            }
          )
        ).send()
      )
    } catch (error) {
      throw error
    }
  }

  static async createDummyChannelTicket(
    this: HoprEthereum,
    counterParty: AccountId,
    challenge: Hash,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    const channelId = await this.utils.getId(
      await this.utils.pubKeyToAccountId(this.self.onChainKeyPair.publicKey),
      counterParty
    )

    // const account = await this.coreConnector.utils.pubKeyToAccountId(counterparty)
    // const { hashedSecret } = await this.coreConnector.hoprChannels.methods.accounts(u8aToHex(account)).call()

    const winProb = new Uint8ArrayE(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
    // const channelId = await this.channelId

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset,
      },
      {
        channelId,
        challenge,
        // @TODO set this dynamically
        epoch: new TicketEpoch(0),
        amount: new Balance(0),
        winProb,
        onChainSecret: new Uint8ArrayE(randomBytes(Hash.SIZE)),
      }
    )

    await this.utils.sign(await ticket.hash, this.self.privateKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset,
    })

    return signedTicket
  }

  static async create(
    this: HoprEthereum,
    counterpartyPubKey: Uint8Array,
    _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    const counterparty = await this.utils.pubKeyToAccountId(counterpartyPubKey)
    let channel: Channel
    let signedChannel: SignedChannel

    if (!this._onChainValuesInitialized) {
      await this.initOnchainValues()
    }

    if (await this.channel.isOpen(counterpartyPubKey)) {
      const record = await this.db.get(Buffer.from(this.dbKeys.Channel(counterpartyPubKey)))
      signedChannel = new SignedChannel({
        bytes: record.buffer,
        offset: record.byteOffset,
      })
      channel = new Channel(this, counterpartyPubKey, signedChannel)
    } else if (sign != null && channelBalance != null) {
      let amount: Balance
      if (this.utils.isPartyA(this.account, counterparty)) {
        amount = channelBalance.balance_a
      } else {
        amount = new Balance(channelBalance.balance.sub(channelBalance.balance_a))
      }

      await this.channel.increaseFunds(counterparty, amount)

      signedChannel = await sign(channelBalance)

      channel = new Channel(this, counterpartyPubKey, signedChannel)

      await waitForConfirmation(
        (
          await this.signTransaction(this.hoprChannels.methods.openChannel(counterparty.toHex()), {
            from: this.account.toHex(),
            to: this.hoprChannels.options.address,
            nonce: await this.nonce,
          })
        ).send()
      )

      await this.db.put(Buffer.from(this.dbKeys.Channel(counterpartyPubKey)), Buffer.from(signedChannel))
    } else {
      throw Error('Invalid input parameters.')
    }

    return channel
  }

  static getAll<T, R>(
    this: HoprEthereum,
    onData: (channel: Channel) => Promise<T>,
    onEnd: (promises: Promise<T>[]) => R
  ): Promise<R> {
    const promises: Promise<T>[] = []
    return new Promise<R>((resolve, reject) => {
      this.db
        .createReadStream({
          gte: Buffer.from(this.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00))),
          lte: Buffer.from(this.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff))),
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = new SignedChannel({
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          promises.push(onData(new Channel(this, this.dbKeys.ChannelKeyParse(key), signedChannel)))
        })
        .on('end', () => resolve(onEnd(promises)))
    })
  }

  static async closeChannels(this: HoprEthereum): Promise<Balance> {
    const result = new BN(0)

    return this.channel.getAll(
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

  static handleOpeningRequest(this: HoprEthereum): (source: AsyncIterable<Uint8Array>) => AsyncIterable<Uint8Array> {
    return (source: AsyncIterable<Uint8Array>) =>
      async function* () {
        for await (const _msg of source) {
          const msg = _msg.slice()
          const signedChannel = new SignedChannel({
            bytes: msg.buffer,
            offset: msg.byteOffset,
          })

          const counterpartyPubKey = await signedChannel.signer
          const counterparty = new AccountId(await this.utils.pubKeyToAccountId(counterpartyPubKey))
          const channelBalance = signedChannel.channel.balance

          if (this.utils.isPartyA(this.account, counterparty)) {
            if (channelBalance.balance.sub(channelBalance.balance_a).gtn(0)) {
              await this.channel.increaseFunds(
                counterparty,
                new Balance(channelBalance.balance.sub(channelBalance.balance_a))
              )
            }
          } else {
            if (channelBalance.balance_a.gtn(0)) {
              await this.channel.increaseFunds(counterparty, channelBalance.balance_a)
            }
          }

          // listen for opening event and update DB
          onceOpen(this, this.account, counterparty).then(() => onOpen(this, counterpartyPubKey, signedChannel))

          yield signedChannel.toU8a()
        }
      }.call(this)
  }
}

export default Channel
