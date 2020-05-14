import BN from 'bn.js'
import chalk from 'chalk'

import PeerId from 'peer-id'
const RELAY_FEE = 10

function fromWei(arg: any, unit: any) {
  return arg.toString()
}

import { pubKeyToPeerId } from '../../utils'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'

import {
  Header,
  deriveTicketKey,
  deriveTicketKeyBlinding,
  deriveTagParameters,
  deriveTicketLastKey,
  deriveTicketLastKeyBlinding,
} from './header'
import { Challenge } from './challenge'
import { PacketTag } from '../../dbKeys'
import Message from './message'
import { LevelUp } from 'levelup'

import Hopr from '../../'

import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface'

/**
 * Encapsulates the internal representation of a packet
 */
export class Packet<Chain extends HoprCoreConnector> extends Uint8Array {
  private _targetPeerId?: PeerId
  private _senderPeerId?: PeerId

  private _header?: Header<Chain>
  private _ticket?: Types.SignedTicket<Types.Ticket, Types.Signature>
  private _challenge?: Challenge<Chain>
  private _message?: Message

  private node: Hopr<Chain>

  constructor(
    node: Hopr<Chain>,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      header: Header<Chain>
      ticket: Types.SignedTicket<Types.Ticket, Types.Signature>
      challenge: Challenge<Chain>
      message: Message
    }
  ) {
    if (arr == null && struct == null) {
      throw Error(`Invalid constructor parameters.`)
    }
    if (arr == null) {
      super(Packet.SIZE(node.paymentChannels))
    } else {
      super(arr.bytes, arr.offset, Packet.SIZE(node.paymentChannels))
    }

    if (struct != null) {
      this.set(struct.header, this.headerOffset - this.byteOffset)
      this.set(struct.ticket, this.ticketOffset - this.byteOffset)
      this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      this.set(struct.message, this.messageOffset - this.byteOffset)

      this._header = struct.header
      this._ticket = struct.ticket
      this._challenge = struct.challenge
      this._message = struct.message
    }

    this.node = node
  }

  subarray(begin: number = 0, end: number = Packet.SIZE(this.node.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get headerOffset(): number {
    return this.byteOffset
  }

  get header(): Header<Chain> {
    if (this._header == null) {
      this._header = new Header<Chain>({ bytes: this.buffer, offset: this.headerOffset })
    }

    return this._header
  }

  get ticketOffset(): number {
    return this.byteOffset + Header.SIZE
  }

  get ticket(): Types.SignedTicket<Types.Ticket, Types.Signature> {
    if (this._ticket == null) {
      this._ticket = this.node.paymentChannels.types.SignedTicket.create({
        bytes: this.buffer,
        offset: this.ticketOffset,
      })
    }

    return this._ticket
  }

  get challengeOffset() {
    return this.byteOffset + Header.SIZE + this.node.paymentChannels.types.SignedTicket.SIZE
  }

  get challenge(): Challenge<Chain> {
    if (this._challenge == null) {
      this._challenge = new Challenge<Chain>(this.node.paymentChannels, {
        bytes: this.buffer,
        offset: this.challengeOffset,
      })
    }

    return this._challenge
  }

  get messageOffset(): number {
    return (
      this.byteOffset +
      Header.SIZE +
      this.node.paymentChannels.types.SignedTicket.SIZE +
      Challenge.SIZE(this.node.paymentChannels)
    )
  }

  get message(): Message {
    if (this._message == null) {
      this._message = new Message(true, {
        bytes: this.buffer,
        offset: this.messageOffset,
      })
    }

    return this._message
  }

  static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain) {
    return (
      Header.SIZE +
      hoprCoreConnector.types.SignedTicket.SIZE +
      Challenge.SIZE(hoprCoreConnector) +
      Message.SIZE
    )
  }

  /**
   * Creates a new packet.
   *
   * @param node the node itself
   * @param msg the message that is sent through the network
   * @param path array of peerId that determines the route that
   * the packet takes
   */
  static async create<Chain extends HoprCoreConnector>(
    node: Hopr<Chain>,
    msg: Uint8Array,
    path: PeerId[]
  ): Promise<Packet<Chain>> {
    const arr = new Uint8Array(Packet.SIZE(node.paymentChannels)).fill(0x00)
    const packet = new Packet<Chain>(node, {
      bytes: arr.buffer,
      offset: arr.byteOffset,
    })

    const { header, secrets, identifier } = await Header.create(node, path, {
      bytes: packet.buffer,
      offset: packet.headerOffset,
    })

    packet._header = header

    const fee = new BN(secrets.length - 1, 10).imul(new BN(RELAY_FEE, 10))

    console.log('---------- New Packet ----------')
    path
      .slice(0, Math.max(0, path.length - 1))
      .forEach((peerId, index) =>
        console.log(`Intermediate ${index} : ${chalk.blue(peerId.toB58String())}`)
      )
    console.log(`Destination    : ${chalk.blue(path[path.length - 1].toB58String())}`)
    console.log('--------------------------------')

    packet._challenge = await Challenge.create(
      node.paymentChannels,
      await node.paymentChannels.utils.hash(deriveTicketKeyBlinding(secrets[0])),
      fee,
      {
        bytes: packet.buffer,
        offset: packet.challengeOffset,
      }
    ).sign(node.peerInfo.id)

    packet._message = Message.create(msg, {
      bytes: packet.buffer,
      offset: packet.messageOffset,
    }).onionEncrypt(secrets)

    const ticketChallenge = await node.paymentChannels.utils.hash(
      secrets.length == 1
        ? u8aConcat(
            deriveTicketLastKey(secrets[0]),
            await node.paymentChannels.utils.hash(deriveTicketLastKeyBlinding(secrets[0]))
          )
        : u8aConcat(
            deriveTicketKey(secrets[0]),
            await node.paymentChannels.utils.hash(deriveTicketKeyBlinding(secrets[1]))
          )
    )

    const channelBalance = node.paymentChannels.types.ChannelBalance.create(undefined, {
      balance: new BN(12345),
      balance_a: new BN(123),
    })

    const channel = await node.paymentChannels.channel.create(
      node.paymentChannels,
      path[0].pubKey.marshal(),
      (_counterparty: Uint8Array) => node.interactions.payments.onChainKey.interact(path[0]),
      channelBalance,
      (_channelBalance: Types.ChannelBalance) =>
        node.interactions.payments.open.interact(path[0], channelBalance)
    )

    packet._ticket = await channel.ticket.create(channel, fee, ticketChallenge, {
      bytes: packet.buffer,
      offset: packet.ticketOffset,
    })

    return packet
  }

  /**
   * Tries to get a previous transaction from the database. If there's no such one,
   * listen to the channel opening event for some time and throw an error if the
   * was not opened within `OPENING_TIMEOUT` ms.
   *
   * @param channelId ID of the channel
   */
  async getPreviousTransaction(channelId: Uint8Array, state) {
    // const recordState = node.paymentChannels.TransactionRecordState
    // switch (state.state) {
    //   case recordState.OPENING:
    //     state = await new Promise((resolve, reject) =>
    //       setTimeout(
    //         (() => {
    //           const eventListener = node.paymentChannels.onceOpened(channelId, resolve)
    //           return () => {
    //             eventListener.removeListener(resolve)
    //             reject(Error(`Sender didn't send payment channel opening request for channel ${chalk.yellow(channelId.toString())} in time.`))
    //           }
    //         })(),
    //         OPENING_TIMEOUT
    //       )
    //     )
    //   case recordState.OPEN:
    //     return state.lastTransaction
    //   default:
    //     throw Error(`Invalid state of payment channel ${chalk.yellow(channelId.toString())}. Got '${state.state}'`)
    // }
  }

  /**
   * Checks the packet and transforms it such that it can be send to the next node.
   *
   * @param node the node itself
   */
  async forwardTransform(): Promise<{
    receivedChallenge: Challenge<Chain>
    ticketKey: Uint8Array
  }> {
    this.header.deriveSecret(this.node.peerInfo.id.privKey.marshal())

    if (await this.testAndSetTag(this.node.db)) {
      throw Error('General error.')
    }

    if (!this.header.verify()) {
      throw Error('General error.')
    }

    this.header.extractHeaderInformation()

    const [sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

    const channelId = await this.node.paymentChannels.utils.getId(
      await this.node.paymentChannels.utils.pubKeyToAccountId(
        this.node.peerInfo.id.pubKey.marshal()
      ),
      await this.node.paymentChannels.utils.pubKeyToAccountId(sender.pubKey.marshal())
    )

    // check if channel exists
    if (
      !(await this.node.paymentChannels.channel.isOpen(
        this.node.paymentChannels,
        new Uint8Array(sender.pubKey.marshal())
      ))
    ) {
      throw Error('Payment channel is not open')
    }

    this.message.decrypt(this.header.derivedSecret)

    const receivedChallenge = this.challenge.getCopy()
    const ticketKey = deriveTicketKeyBlinding(this.header.derivedSecret)

    if (u8aEquals(this.node.peerInfo.id.pubKey.marshal(), this.header.address)) {
      await this.prepareDelivery(null, null, channelId)
    } else {
      await this.prepareForward(null, null, target)
    }

    return { receivedChallenge, ticketKey }
  }

  /**
   * Prepares the delivery of the packet.
   *
   * @param node the node itself
   * @param state current off-chain state
   * @param newState future off-chain state
   * @param nextNode the ID of the payment channel
   */
  async prepareDelivery(state, newState, nextNode): Promise<void> {
    if (
      !u8aEquals(
        await this.node.paymentChannels.utils.hash(
          u8aConcat(
            deriveTicketLastKey(this.header.derivedSecret),
            await this.node.paymentChannels.utils.hash(
              deriveTicketLastKeyBlinding(this.header.derivedSecret)
            )
          )
        ),
        this.ticket.ticket.challenge
      )
    ) {
      throw Error('General error.')
    }
    this.message.encrypted = false

    // const challenges = [secp256k1.publicKeyCreate(Buffer.from(deriveTicketKey(this.header.derivedSecret)))]
    // const previousChallenges = await (await node.paymentChannels.channel.create(node.paymentChannels, nextNode)).getPreviousChallenges()
    // if (previousChallenges != null) challenges.push(Buffer.from(previousChallenges))
    // if (state.channelKey) challenges.push(secp256k1.publicKeyCreate(state.channelKey))
    // if (!this.ticket.curvePoint.equals(secp256k1.publicKeyCombine(challenges))) {
    //   throw Error('General error.')
    // }
    // newState.channelKey = secp256k1.privateKeyTweakAdd(
    //   state.channelKey || Buffer.alloc(PRIVATE_KEY_LENGTH, 0),
    //   Buffer.from(deriveTicketKey(this.header.derivedSecret))
    // )
    // await node.paymentChannels.setState(nextNode, newState)
  }

  /**
   * Prepares the packet in order to forward it to the next node.
   *
   * @param node the node itself
   * @param state current off-chain state
   * @param newState future off-chain state
   * @param channelId the ID of the payment channel
   * @param target peer Id of the next node
   */
  async prepareForward(state, newState, target: PeerId): Promise<void> {
    if (
      !u8aEquals(
        await this.node.paymentChannels.utils.hash(
          u8aConcat(deriveTicketKey(this.header.derivedSecret), this.header.hashedKeyHalf)
        ),
        this.ticket.ticket.challenge
      )
    ) {
      throw Error('General error.')
    }

    const channelId = await this.node.paymentChannels.utils.getId(
      await this.node.paymentChannels.utils.pubKeyToAccountId(
        this.node.peerInfo.id.pubKey.marshal()
      ),
      await this.node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal())
    )

    await this.node.db.put(
      Buffer.from(
        this.node.dbKeys.UnAcknowledgedTickets(target.pubKey.marshal(), this.header.hashedKeyHalf)
      ),
      Buffer.from(this.ticket)
    )

    // const challenges = [secp256k1.publicKeyCreate(Buffer.from(deriveTicketKey(this.header.derivedSecret))), this.header.hashedKeyHalf]
    // let previousChallenges = await (await node.paymentChannels.channel.create(node.paymentChannels, await node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()))).getPreviousChallenges()

    // if (previousChallenges != null) {
    //   challenges.push(previousChallenges)
    // }

    // if (state.channelKey) {
    //   challenges.push(secp256k1.publicKeyCreate(state.channelKey))
    // }

    // if (!this.ticket.curvePoint.equals(secp256k1.publicKeyCombine(challenges.map((challenge: Uint8Array) => Buffer.from(challenge))))) {
    //   throw Error('General error.')
    // }

    const channelBalance = this.node.paymentChannels.types.ChannelBalance.create(undefined, {
      balance: new BN(12345),
      balance_a: new BN(123),
    })

    const channel = await this.node.paymentChannels.channel.create(
      this.node.paymentChannels,
      target.pubKey.marshal(),
      (_counterparty: Uint8Array) => this.node.interactions.payments.onChainKey.interact(target),
      channelBalance,
      (_channelBalance: Types.ChannelBalance) =>
        this.node.interactions.payments.open.interact(target, channelBalance)
    )

    const receivedMoney = this.ticket.ticket.getEmbeddedFunds()

    this.node.log(
      `Received ${chalk.magenta(
        `${fromWei(receivedMoney, 'ether').toString()} ETH`
      )} on channel ${chalk.yellow(channelId.toString())}.`
    )

    // if (receivedMoney.lt(RELAY_FEE)) {
    //   throw Error('Bad transaction.')
    // }

    this.header.transformForNextNode()

    const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

    this._challenge = await Challenge.create<Chain>(
      this.node.paymentChannels,
      // @TODO use correct value
      this.header.hashedKeyHalf,
      forwardedFunds,
      {
        bytes: this.buffer,
        offset: this.challengeOffset,
      }
    ).sign(this.node.peerInfo.id)

    // const [tx] = await Promise.all([
    //   node.paymentChannels.transfer(await node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()), forwardedFunds, this.header.encryptionKey),
    //   node.paymentChannels.setState(channelId, newState),
    //   node.db
    //     .batch()
    //     .put(node.paymentChannels.dbKeys.ChannelId(await this.challenge.signatureHash), channelId)
    //     .put(node.paymentChannels.dbKeys.Challenge(channelId, this.header.hashedKeyHalf), deriveTicketKey(this.header.derivedSecret))
    //     .write()
    // ])

    this._ticket = await channel.ticket.create(channel, forwardedFunds, this.header.encryptionKey, {
      bytes: this.buffer,
      offset: this.ticketOffset,
    })
  }

  /**
   * Computes the peerId of the next downstream node and caches it for later use.
   */
  async getTargetPeerId(): Promise<PeerId> {
    if (this._targetPeerId !== undefined) {
      return this._targetPeerId
    }

    this._targetPeerId = await pubKeyToPeerId(this.header.address)

    return this._targetPeerId
  }

  /**
   * Computes the peerId if the preceeding node and caches it for later use.
   */
  async getSenderPeerId(): Promise<PeerId> {
    if (this._senderPeerId !== undefined) {
      return this._senderPeerId
    }

    this._senderPeerId = await pubKeyToPeerId(await this.ticket.signer)

    return this._senderPeerId
  }

  /**
   * Checks whether the packet has already been seen.
   */
  async testAndSetTag(db: LevelUp): Promise<boolean> {
    const key = PacketTag(deriveTagParameters(this.header.derivedSecret))

    try {
      await db.get(key)
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound === undefined || !err.notFound) {
        await db.put(Buffer.from(key), Buffer.from(''))
        return
      }
    }

    throw Error('Key is already present. Cannot accept packet because it might be a duplicate.')
  }
}
