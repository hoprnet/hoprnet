import type { Channel as IChannel, Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
// import Web3 from "web3"
import { SignedChannel, Moment, Hash, AccountId, ChannelId, Balance, ChannelBalance, Ticket, State, Public, TicketEpoch } from '../types'
import { ChannelStatus } from '../types/channel'
import { HASH_LENGTH } from '../constants'
import { u8aToHex, u8aXOR, stringToU8a, u8aEquals } from '../core/u8a'
import { waitForConfirmation, waitFor, hash, getId, stateCountToStatus } from '../utils'
import { HoprChannels as IHoprChannels } from '../tsc/web3/HoprChannels'
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
  const events: ReturnType<IHoprChannels['events']['OpenedChannel']>[] = []

  return new Promise<{
    opener: string
    counterParty: string
  }>((resolve, reject) => {
    const onData = async (data: any) => {
      const { opener, counterParty } = data.returnValues
      const _channelId = await coreConnector.utils.getId(stringToU8a(opener), stringToU8a(counterParty))

      if (!u8aEquals(_channelId, channelId)) {
        return
      }

      resolve(data.returnValues)
    }
    const onError = (error: any) => reject(error)

    events.push(
      coreConnector.hoprChannels.events
        .OpenedChannel({
          filter: {
            opener: self.toHex(),
            counterParty: counterparty.toHex()
          }
        })
        .on('data', onData)
        .on('error', onError)
    )

    events.push(
      coreConnector.hoprChannels.events
        .OpenedChannel({
          filter: {
            opener: counterparty.toHex(),
            counterParty: self.toHex()
          }
        })
        .on('data', onData)
        .on('error', onError)
    )
  }).finally(() => {
    events.forEach(event => event.removeAllListeners())
  })
}

const onceClosed = async (coreConnector: HoprEthereum, self: AccountId, counterparty: AccountId) => {
  const channelId = await getId(self, counterparty)
  const events: ReturnType<IHoprChannels['events']['ClosedChannel']>[] = []

  return new Promise<{
    closer: string
    counterParty: string
  }>((resolve, reject) => {
    const onData = async (data: any) => {
      const { closer, counterParty } = data.returnValues
      const _channelId = await coreConnector.utils.getId(stringToU8a(closer), stringToU8a(counterParty))

      if (!u8aEquals(_channelId, channelId)) {
        return
      }

      resolve(data.returnValues)
    }
    const onError = (error: any) => reject(error)

    events.push(
      coreConnector.hoprChannels.events
        .ClosedChannel({
          filter: {
            opener: self.toHex(),
            counterParty: counterparty.toHex()
          }
        })
        .on('data', onData)
        .on('error', onError)
    )

    events.push(
      coreConnector.hoprChannels.events
        .ClosedChannel({
          filter: {
            opener: counterparty.toHex(),
            counterParty: self.toHex()
          }
        })
        .on('data', onData)
        .on('error', onError)
    )
  }).finally(() => {
    events.forEach(event => event.removeAllListeners())
  })
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
    this.status.then(status => {
      if (status === ChannelStatus.UNINITIALISED) {
        console.log('found channel off-chain but its closed on-chain')
        this.onClose()
      }
    })

    // if channel is closed
    this.onceClosed().then(async () => {
      return this.onClose()
    })
  }

  private async onceOpen() {
    return onceOpen(
      this.coreConnector,
      this.coreConnector.account,
      await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
    )
  }

  private async onceClosed() {
    return onceClosed(
      this.coreConnector,
      this.coreConnector.account,
      await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
    )
  }

  private async onOpen(): Promise<void> {
    return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
  }

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
    return this._signedChannel.signer
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

        return resolve(new State(undefined, {
          // @TODO: implement this once on-chain channel secrets are added
          secret: new Hash(new Uint8Array(Hash.SIZE).fill(0x0)),
          // not needed
          pubkey: new Public(new Uint8Array(Public.SIZE).fill(0x0)),
          epoch: new TicketEpoch(status)
        }))
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
        throw Error("channel must be 'open' or 'pending'")
      }

      if (status === ChannelStatus.OPEN) {
        await waitForConfirmation(
          this.coreConnector.hoprChannels.methods
            .initiateChannelClosure(u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
            .send({
              from: this.coreConnector.account.toHex(),
              gas: 200e3
            })
        )

        channel = await getChannel(this.coreConnector, await this.channelId)

        // TODO: update to handle localnet & mainnet
        await waitFor({
          getCurrentBlock: () => {
            return this.coreConnector.web3.eth.getBlockNumber().then(blockNumber => {
              return this.coreConnector.web3.eth.getBlock(blockNumber)
            })
          },
          web3: this.coreConnector.web3,
          timestamp: Number(channel.closureTime) * 1e3
        })

        await waitForConfirmation(
          this.coreConnector.hoprChannels.methods
            .claimChannelClosure(u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
            .send({
              from: this.coreConnector.account.toHex(),
              gas: 200e3
            })
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
          gt: Buffer.from(
            this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(HASH_LENGTH).fill(0x00))
          ),
          lt: Buffer.from(
            this.coreConnector.dbKeys.Challenge(await this.channelId, new Uint8Array(HASH_LENGTH).fill(0xff))
          )
        })
        .on('error', reject)
        .on('data', ({ key, ownKeyHalf }) => {
          const challenge = this.coreConnector.dbKeys.ChallengeKeyParse(key)[1]

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
    const channelId = await coreConnector.utils.getId(coreConnector.account, counterparty).then(res => new Hash(res))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      getChannel(coreConnector, channelId).then(channel => {
        const state = Number(channel.stateCounter) % 10
        return state === ChannelStatus.OPEN || state === ChannelStatus.PENDING
      }),
      coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Channel(counterpartyPubKey))).then(
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
        console.log(`Channel ${u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`)
        await onClose(coreConnector, counterpartyPubKey)
      } else {
        throw Error(`Channel ${u8aToHex(channelId)} exists on-chain but not off-chain.`)
      }
    }

    return onChain && offChain
  }

  static async increaseFunds(
    hoprEthereum: HoprEthereum,
    spender: AccountId,
    counterparty: AccountId,
    amount: Balance
  ): Promise<void> {
    try {
      if ((await hoprEthereum.accountBalance).lt(amount)) {
        throw Error('Insufficient funds.')
      }

      const allowance = await hoprEthereum.hoprToken.methods
        .allowance(hoprEthereum.account.toHex(), spender.toHex())
        .call()
        .then(v => new BN(v))

      if (allowance.isZero()) {
        await waitForConfirmation(
          hoprEthereum.hoprToken.methods.approve(spender.toHex(), amount.toString()).send({
            from: hoprEthereum.account.toHex(),
            gas: 200e3
          })
        )
      } else if (allowance.lt(amount)) {
        await waitForConfirmation(
          hoprEthereum.hoprToken.methods.increaseAllowance(spender.toHex(), amount.sub(allowance).toString()).send({
            from: hoprEthereum.account.toHex(),
            gas: 200e3
          })
        )
      }

      await waitForConfirmation(
        hoprEthereum.hoprChannels.methods
          .fundChannel(hoprEthereum.account.toHex(), counterparty.toHex(), amount.toString())
          .send({
            from: hoprEthereum.account.toHex(),
            gas: 200e3
          })
      )
    } catch (error) {
      throw error
    }
  }

  static async create(
    hoprEthereum: HoprEthereum,
    counterpartyPubKey: Uint8Array,
    _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    const counterparty = new AccountId(await hoprEthereum.utils.pubKeyToAccountId(counterpartyPubKey))
    let channel: Channel
    let signedChannel: SignedChannel

    if (await this.isOpen(hoprEthereum, counterpartyPubKey)) {
      const record = await hoprEthereum.db.get(Buffer.from(hoprEthereum.dbKeys.Channel(counterpartyPubKey)))
      signedChannel = new SignedChannel({
        bytes: record.buffer,
        offset: record.byteOffset
      })
      channel = new Channel(hoprEthereum, counterpartyPubKey, signedChannel)
    } else if (sign != null && channelBalance != null) {
      const spender = hoprEthereum.hoprChannels.options.address

      let amount: Balance
      if (hoprEthereum.utils.isPartyA(hoprEthereum.account, counterparty)) {
        amount = channelBalance.balance_a
      } else {
        amount = new Balance(channelBalance.balance.sub(channelBalance.balance_a))
      }

      await Channel.increaseFunds(hoprEthereum, new AccountId(stringToU8a(spender)), counterparty, amount)

      signedChannel = await sign(channelBalance)
      channel = new Channel(hoprEthereum, counterpartyPubKey, signedChannel)

      // TODO: use 'fundChannelWithSig'
      // await waitForConfirmation(
      //   hoprEthereum.hoprChannels.methods.fundChannelWithSig(
      //     (await channel.channel).stateCounter,
      //     channelBalance.balance.toString(),
      //     channelBalance.balance_a.toString(),
      //     String(Math.floor(+new Date() / 1e3) + (60 * 60 * 24)), // TODO: improve this
      //     signatureParameters.r,
      //     signatureParameters.s,
      //     "0x1b"
      //   ).send({
      //     from: hoprEthereum.account.toHex(),
      //     gas: 200e3
      //   })
      // )

      await waitForConfirmation(
        hoprEthereum.hoprChannels.methods.openChannel(counterparty.toHex()).send({
          from: hoprEthereum.account.toHex(),
          gas: 200e3
        })
      )

      await hoprEthereum.db.put(
        Buffer.from(hoprEthereum.dbKeys.Channel(counterpartyPubKey)),
        Buffer.from(signedChannel)
      )
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
          gt: Buffer.from(hoprEthereum.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00))),
          lt: Buffer.from(hoprEthereum.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff)))
        })
        .on('error', err => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = new SignedChannel({
            bytes: value.buffer,
            offset: value.byteOffset
          })

          promises.push(onData(new Channel(hoprEthereum, hoprEthereum.dbKeys.ChannelKeyParse(key), signedChannel)))
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

  static handleOpeningRequest(
    hoprEthereum: HoprEthereum
  ): (source: AsyncIterable<Uint8Array>) => AsyncIterator<Uint8Array> {
    return source => {
      return (async function*() {
        for await (const _msg of source) {
          const msg = _msg.slice()
          const signedChannel = new SignedChannel({
            bytes: msg.buffer,
            offset: msg.byteOffset
          })

          const counterpartyPubKey = await signedChannel.signer
          const counterparty = new AccountId(await hoprEthereum.utils.pubKeyToAccountId(counterpartyPubKey))
          const channelBalance = signedChannel.channel.balance
          // const channelId = await hoprEthereum.utils.getId(hoprEthereum.account, counterparty)
          const spender = hoprEthereum.hoprChannels.options.address

          if (hoprEthereum.utils.isPartyA(hoprEthereum.account, counterparty)) {
            await Channel.increaseFunds(
              hoprEthereum,
              new AccountId(stringToU8a(spender)),
              counterparty,
              channelBalance.balance_a
            )
          } else {
            await Channel.increaseFunds(
              hoprEthereum,
              new AccountId(stringToU8a(spender)),
              counterparty,
              new Balance(channelBalance.balance.sub(channelBalance.balance_a))
            )
          }

          // listen for opening event and update DB
          onceOpen(hoprEthereum, hoprEthereum.account, counterparty)
            .then(() => onOpen(hoprEthereum, counterpartyPubKey, signedChannel))

          yield signedChannel.toU8a()
        }
      })()
    }
  }
}

export default Channel

// TODO: remove this
// const getSignatureParameters = (signature: string) => {
//   const r = signature.slice( 0, 66 );
//   const s = `0x${signature.slice( 66, 130 )}`;
//   const v = `0x${signature.slice( 130, 132 )}`;
//   let vN = Web3.utils.hexToNumber(v)

//   if ( ![ 27, 28 ].includes( vN ) ) vN += 27;

//   return {
//       r,
//       s,
//       v: vN
//   };
// };

// const onceFundedByCounterparty = (
//   coreConnector: HoprEthereum,
//   channelId: Hash,
//   counterparty: AccountId
// ): Promise<void> => {
//   let event: ReturnType<IHoprChannels['events']['FundedChannel']>

//   return new Promise<void>((resolve, reject) => {
//     // TODO: better to filter
//     event = coreConnector.hoprChannels.events
//       .FundedChannel()
//       .on('data', async data => {
//         const { recipient, counterParty: _counterparty } = data.returnValues
//         const _channelId = await coreConnector.utils.getId(stringToU8a(recipient), stringToU8a(_counterparty))

//         if (!u8aEquals(_channelId, channelId)) {
//           return
//         }
//         if (!u8aEquals(stringToU8a(_counterparty), counterparty)) {
//           return
//         }

//         resolve()
//       })
//       .on('error', error => {
//         reject(error)
//       })
//   }).finally(() => {
//     event.removeAllListeners()
//   })
// }
