import BN from 'bn.js'
import { blue, green } from 'chalk'
import PeerId from 'peer-id'
import { u8aConcat, u8aEquals, u8aToHex, pubKeyToPeerId, serializeToU8a } from '@hoprnet/hopr-utils'
import { getTickets, validateUnacknowledgedTicket, validateCreatedTicket } from '../../utils/tickets'
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
export class Packet<Chain extends HoprCoreConnector> {
  private node: Hopr<Chain>

  constructor(
    private peerId: PeerId, // of this node
    private header: Header,

    private ticket: Types.SignedTicket,
    private challenge: Challenge,
    private message: Message,
    private sizeofSignedTicket: number,
    private sizeofChallenge: number
  ) {}

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.header, Header.SIZE],
      [this.ticket, this.sizeofSignedTicket],
      [this.challenge, this.sizeofChallenge],
      [this.message, Message.SIZE] 
    ])
  }

  public static async deserialize<Chain extends HoprCoreConnector>(
    peerId: PeerId,
    arr: Uint8Array,
    sizeofSignedTicket: number,
  ): Promise<Packet<Chain>> {
    let i = arr.byteOffset
    const header = new Header({ bytes: arr.buffer, offset: i })
    i += Header.SIZE
    const ticket = await this.node.paymentChannels.types.SignedTicket.create({
      bytes: arr.buffer,
      offset: i
    })
    i += this.node.paymentChannels.types.SignedTicket.SIZE
    const challenge = new Challenge<Chain>(this.node.paymentChannels, {
      bytes: arr.buffer,
      offset: i
    })
    return new Packet(peerId, header, ticket, challenge)
  }

  /*
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
*/

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
    const chain = node.paymentChannels
    const { Balance } = chain.types

    const arr = new Uint8Array(Packet.SIZE(chain)).fill(0x00)
    const packet = new Packet<Chain>(node, {
      bytes: arr.buffer,
      offset: arr.byteOffset
    })

    const { header, secrets } = await Header.create(node, path, {
      bytes: packet.buffer,
      offset: packet.headerOffset
    })

    packet._header = header

    const fee = new BN(secrets.length - 1).imul(new BN(node.ticketAmount))

    log('---------- New Packet ----------')
    path
      .slice(0, Math.max(0, path.length - 1))
      .forEach((peerId, index) => log(`Intermediate ${index} : ${blue(peerId.toB58String())}`))
    log(`Destination    : ${blue(path[path.length - 1].toB58String())}`)
    log('--------------------------------')

    packet._challenge = await Challenge.create(
      chain,
      await chain.utils.hash(deriveTicketKeyBlinding(secrets[0])),
      fee,
      {
        bytes: packet.buffer,
        offset: packet.challengeOffset
      }
    ).sign(node.getPeerId())

    packet._message = Message.create(msg, {
      bytes: packet.buffer,
      offset: packet.messageOffset
    }).onionEncrypt(secrets)

    const ticketChallenge = await chain.utils.hash(
      secrets.length == 1
        ? deriveTicketLastKey(secrets[0])
        : await chain.utils.hash(
            u8aConcat(deriveTicketKey(secrets[0]), await chain.utils.hash(deriveTicketKeyBlinding(secrets[1])))
          )
    )

    if (secrets.length > 1) {
      log(`before creating channel`)

      const channel = await chain.channel.create(path[0].pubKey.marshal(), (_counterparty: Uint8Array) =>
        node._interactions.payments.onChainKey.interact(path[0])
      )

      packet.ticket = await channel.ticket.create(new Balance(fee), ticketChallenge, node.ticketWinProb, {
        bytes: packet.buffer,
        offset: packet.ticketOffset
      })

      const myAccountId = await chain.utils.pubKeyToAccountId(node.getId().pubKey.marshal())
      const counterpartyAccountId = await chain.utils.pubKeyToAccountId(channel.counterparty)
      const amPartyA = chain.utils.isPartyA(myAccountId, counterpartyAccountId)
      await validateCreatedTicket({
        myBalance: await (amPartyA ? channel.balance_a : channel.balance_b),
        signedTicket: packet._ticket
      })
    } else if (secrets.length == 1) {
      packet.ticket = await chain.channel.createDummyChannelTicket(
        await chain.utils.pubKeyToAccountId(path[0].pubKey.marshal()),
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
    this.header.deriveSecret(this.peerId.privKey.marshal())

    if (await this.testAndSetTag(this.node.db)) {
      verbose('Error setting tag')
      throw Error('Error setting tag')
    }

    if (!this.header.verify()) {
      verbose('Error verifying header', this.header)
      throw Error('Error verifying header')
    }

    this.header.extractHeaderInformation()

    let isRecipient = u8aEquals(this.peerId.pubKey.marshal(), this.header.address)

    let sender: PeerId, target: PeerId
    if (!isRecipient) {
      ;[sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

      try {
        await validateUnacknowledgedTicket({
          node: this.node,
          senderPeerId: sender,
          signedTicket: await this.ticket,
          getTickets: () =>
            getTickets(this.node, {
              signer: sender.pubKey.marshal()
            })
        })
      } catch (error) {
        verbose('Could not validate unacknowledged ticket', error.message)
        throw error
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
  async prepareForward(_originalSender: PeerId, target: PeerId): Promise<void> {
    const chain = this.node.paymentChannels
    const { Balance, ChannelBalance } = chain.types
    const signedTicket = await this.ticket
    const ticket = signedTicket.ticket
    const sender = this.node.getId()
    const senderAccountId = await chain.utils.pubKeyToAccountId(sender.pubKey.marshal())
    const targetAccountId = await chain.utils.pubKeyToAccountId(target.pubKey.marshal())
    const amPartyA = chain.utils.isPartyA(senderAccountId, targetAccountId)
    const challenge = u8aConcat(deriveTicketKey(this.header.derivedSecret), this.header.hashedKeyHalf)

    if (!u8aEquals(await chain.utils.hash(await chain.utils.hash(challenge)), ticket.challenge)) {
      verbose('Error preparing to forward')
      throw Error('Error preparing forward')
    }

    const unacknowledged = new UnacknowledgedTicket(chain, undefined, {
      signedTicket,
      secretA: deriveTicketKey(this.header.derivedSecret)
    })

    log(
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${green(
        u8aToHex(this.header.hashedKeyHalf)
      )} from ${blue(target.toB58String())}`
    )
    await this.node.db.put(
      Buffer.from(this.node._dbKeys.UnAcknowledgedTickets(this.header.hashedKeyHalf)),
      Buffer.from(unacknowledged)
    )

    // get new ticket amount
    const fee = new Balance(ticket.amount.isub(new BN(this.node.ticketAmount)))

    if (fee.gtn(0)) {
      const channelBalance = ChannelBalance.create(undefined, {
        balance: fee,
        balance_a: amPartyA ? fee : new BN(0)
      })

      const channel = await chain.channel.create(
        target.pubKey.marshal(),
        (_counterparty: Uint8Array) => this.node._interactions.payments.onChainKey.interact(target),
        channelBalance,
        (_channelBalance: Types.ChannelBalance) =>
          this.node._interactions.payments.open.interact(target, channelBalance)
      )

      this.ticket = await channel.ticket.create(fee, this.header.encryptionKey, this.node.ticketWinProb)

      await validateCreatedTicket({
        myBalance: await (amPartyA ? channel.balance_a : channel.balance_b),
        signedTicket: this.ticket
      })
    } else if (fee.isZero()) {
      this.ticket = await chain.channel.createDummyChannelTicket(
        await chain.utils.pubKeyToAccountId(target.pubKey.marshal()),
        this.header.encryptionKey
      )
    } else {
      throw Error(`Cannot forward packet`)
    }

    this.header.transformForNextNode()
    this.challenge = await Challenge.create<Chain>(chain, this.header.hashedKeyHalf, fee).sign(sender)
  }

  /**
   * Computes the peerId of the next downstream node.
   */
  async getTargetPeerId(): Promise<PeerId> {
    return await pubKeyToPeerId(this.header.address)
  }

  /**
   * Computes the peerId if the preceeding node
   */
  async getSenderPeerId(): Promise<PeerId> {
    return await pubKeyToPeerId(await (await this.ticket).signer)
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
        return false
      } else {
        throw err
      }
    }

    throw Error('Key is already present. Cannot accept packet because it might be a duplicate.')
  }
}
