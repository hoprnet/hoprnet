import BN from 'bn.js'
import chalk from 'chalk'

import PeerId from 'peer-id'
const RELAY_FEE = 1

import { pubKeyToPeerId } from '../../utils'
import { u8aConcat, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

import { Header, deriveTicketKey, deriveTicketKeyBlinding, deriveTagParameters, deriveTicketLastKey } from './header'
import { Challenge } from './challenge'
import { PacketTag } from '../../dbKeys'
import Message from './message'
import { LevelUp } from 'levelup'

import Debug from 'debug'

import Hopr from '../../'

import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface'
import { UnacknowledgedTicket } from '../ticket'

const log = Debug('hopr-core:message:packet')
const verbose = Debug('hopr-core:verbose:message:packet')

/**
 * Encapsulates the internal representation of a packet
 */
export class Packet<Chain extends HoprCoreConnector> extends Uint8Array {
  private _targetPeerId?: PeerId
  private _senderPeerId?: PeerId

  private _header?: Header<Chain>
  private _ticket?: Types.SignedTicket
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
      ticket: Types.SignedTicket
      challenge: Challenge<Chain>
      message: Message
    }
  ) {
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

  slice(begin: number = 0, end: number = Packet.SIZE(this.node.paymentChannels)) {
    return this.subarray(begin, end)
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

  get ticket(): Promise<Types.SignedTicket> {
    if (this._ticket != null) {
      return Promise.resolve(this._ticket)
    }

    return new Promise<Types.SignedTicket>(async (resolve) => {
      this._ticket = await this.node.paymentChannels.types.SignedTicket.create({
        bytes: this.buffer,
        offset: this.ticketOffset
      })

      resolve(this._ticket)
    })
  }

  get challengeOffset() {
    return this.byteOffset + Header.SIZE + this.node.paymentChannels.types.SignedTicket.SIZE
  }

  get challenge(): Challenge<Chain> {
    if (this._challenge == null) {
      this._challenge = new Challenge<Chain>(this.node.paymentChannels, {
        bytes: this.buffer,
        offset: this.challengeOffset
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
        offset: this.messageOffset
      })
    }

    return this._message
  }

  static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain) {
    return Header.SIZE + hoprCoreConnector.types.SignedTicket.SIZE + Challenge.SIZE(hoprCoreConnector) + Message.SIZE
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
      offset: arr.byteOffset
    })

    const { header, secrets } = await Header.create(node, path, {
      bytes: packet.buffer,
      offset: packet.headerOffset
    })

    packet._header = header

    const fee = new BN(secrets.length - 1, 10).imul(new BN(RELAY_FEE, 10))

    log('---------- New Packet ----------')
    path
      .slice(0, Math.max(0, path.length - 1))
      .forEach((peerId, index) => log(`Intermediate ${index} : ${chalk.blue(peerId.toB58String())}`))
    log(`Destination    : ${chalk.blue(path[path.length - 1].toB58String())}`)
    log('--------------------------------')

    packet._challenge = await Challenge.create(
      node.paymentChannels,
      await node.paymentChannels.utils.hash(deriveTicketKeyBlinding(secrets[0])),
      fee,
      {
        bytes: packet.buffer,
        offset: packet.challengeOffset
      }
    ).sign(node.peerInfo.id)

    packet._message = Message.create(msg, {
      bytes: packet.buffer,
      offset: packet.messageOffset
    }).onionEncrypt(secrets)

    const ticketChallenge = await node.paymentChannels.utils.hash(
      secrets.length == 1
        ? deriveTicketLastKey(secrets[0])
        : await node.paymentChannels.utils.hash(
            u8aConcat(
              deriveTicketKey(secrets[0]),
              await node.paymentChannels.utils.hash(deriveTicketKeyBlinding(secrets[1]))
            )
          )
    )

    if (secrets.length > 1) {
      log(`before creating channel`)

      const channel = await node.paymentChannels.channel.create(path[0].pubKey.marshal(), (_counterparty: Uint8Array) =>
        node._interactions.payments.onChainKey.interact(path[0])
      )

      const newFee = new node.paymentChannels.types.Balance(100)

      // log(Object.getOwnPropertyNames(newFee))
      // @ts-ignore
      packet._ticket = await channel.ticket.create(newFee, ticketChallenge, 1.0, {
        bytes: packet.buffer,
        offset: packet.ticketOffset
      })
    } else if (secrets.length == 1) {
      packet._ticket = await node.paymentChannels.channel.createDummyChannelTicket(
        await node.paymentChannels.utils.pubKeyToAccountId(path[0].pubKey.marshal()),
        ticketChallenge,
        {
          bytes: packet.buffer,
          offset: packet.ticketOffset
        }
      )
    }

    return packet
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
      verbose('Error setting tag')
      throw Error('Error setting tag')
    }

    if (!this.header.verify()) {
      verbose('Error verifying header', this.header)
      throw Error('Error verifying header')
    }

    this.header.extractHeaderInformation()

    let isRecipient = u8aEquals(this.node.peerInfo.id.pubKey.marshal(), this.header.address)

    let sender: PeerId, target: PeerId
    if (!isRecipient) {
      ;[sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

      // check if channel exists

      if (!(await this.node.paymentChannels.channel.isOpen(new Uint8Array(sender.pubKey.marshal())))) {
        throw Error('Payment channel is not open')
      }
    }

    this.message.decrypt(this.header.derivedSecret)

    const receivedChallenge = this.challenge.getCopy()
    const ticketKey = deriveTicketKeyBlinding(this.header.derivedSecret)

    if (isRecipient) {
      await this.prepareDelivery()
    } else {
      await this.prepareForward(sender, target)
    }

    return { receivedChallenge, ticketKey }
  }

  /**
   * Prepares the delivery of the packet.
   */
  async prepareDelivery(): Promise<void> {
    if (
      !u8aEquals(
        await this.node.paymentChannels.utils.hash(deriveTicketLastKey(this.header.derivedSecret)),
        (await this.ticket).ticket.challenge
      )
    ) {
      verbose('Error preparing delivery')
      throw Error('Error preparing delivery')
    }

    this.message.encrypted = false
  }

  /**
   * Prepares the packet in order to forward it to the next node.
   *
   * @param sender peer Id of the previous node
   * @param target peer Id of the next node

   */
  async prepareForward(_sender: PeerId, target: PeerId): Promise<void> {
    if (
      !u8aEquals(
        await this.node.paymentChannels.utils.hash(
          await this.node.paymentChannels.utils.hash(
            u8aConcat(deriveTicketKey(this.header.derivedSecret), this.header.hashedKeyHalf)
          )
        ),
        (await this.ticket).ticket.challenge
      )
    ) {
      verbose('Error preparing to forward')
      throw Error('Error preparing forward')
    }

    const channelIdOutgoing = await this.node.paymentChannels.utils.getId(
      await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()),
      await this.node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal())
    )

    const unacknowledged = new UnacknowledgedTicket(this.node.paymentChannels, undefined, {
      signedTicket: await this.ticket,
      secretA: deriveTicketKey(this.header.derivedSecret)
    })

    log(
      `During forward: storing hashedKey`,
      u8aToHex(this.header.hashedKeyHalf),
      `we are ${this.node.peerInfo.id.toB58String()}`
    )
    await this.node.db.put(
      Buffer.from(this.node._dbKeys.UnAcknowledgedTickets(this.header.hashedKeyHalf)),
      Buffer.from(unacknowledged)
    )

    const receivedMoney = (await this.ticket).ticket.getEmbeddedFunds()

    const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

    if (forwardedFunds.gtn(0)) {
      const channelBalance = this.node.paymentChannels.types.ChannelBalance.create(undefined, {
        balance: new BN(10000),
        balance_a: new BN(5000)
      })

      const channel = await this.node.paymentChannels.channel.create(
        target.pubKey.marshal(),
        (_counterparty: Uint8Array) => this.node._interactions.payments.onChainKey.interact(target),
        channelBalance,
        (_channelBalance: Types.ChannelBalance) => this.node._interactions.payments.open.interact(target, channelBalance)
      )

      this._ticket = await channel.ticket.create(
        // @ts-ignore
        forwardedFunds,
        this.header.encryptionKey,
        1.0,
        {
          bytes: this.buffer,
          offset: this.ticketOffset
        }
      )
    } else if (forwardedFunds.isZero()) {
      this._ticket = await this.node.paymentChannels.channel.createDummyChannelTicket(
        await this.node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()),
        this.header.encryptionKey,
        {
          bytes: this.buffer,
          offset: this.ticketOffset
        }
      )
    } else {
      throw Error(`Cannot forward ${forwardedFunds.toNumber()}`)
    }

    log(
      `Received ${chalk.magenta(
        `${this.node.paymentChannels.utils.convertUnit(receivedMoney, 'wei', 'ether').toString()} ETH`
      )} on channel ${chalk.yellow(channelIdOutgoing.toString())}.`
    )

    this.header.transformForNextNode()

    this._challenge = await Challenge.create<Chain>(
      this.node.paymentChannels,
      this.header.hashedKeyHalf,
      forwardedFunds,
      {
        bytes: this.buffer,
        offset: this.challengeOffset
      }
    ).sign(this.node.peerInfo.id)
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

    this._senderPeerId = await pubKeyToPeerId(await (await this.ticket).signer)

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
