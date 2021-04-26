import BN from 'bn.js'
import LibP2P from 'libp2p'
import { blue, green } from 'chalk'
import PeerId from 'peer-id'
import { u8aConcat, u8aEquals, u8aToHex, pubKeyToPeerId } from '@hoprnet/hopr-utils'
import { getTickets } from '../../utils/tickets'
import { Header, deriveTicketKey, deriveTicketKeyBlinding, deriveTagParameters, deriveTicketLastKey } from './header'
import { Challenge } from './challenge'
import { PacketTag, UnAcknowledgedTickets } from '../../dbKeys'
import Message from './message'
import { LevelUp } from 'levelup'
import Debug from 'debug'
import Hopr from '../../'
import HoprCoreEthereum, {
  Hash,
  PublicKey,
  Ticket,
  Balance,
  UnacknowledgedTicket,
  Channel,
  getWinProbabilityAsFloat
} from '@hoprnet/hopr-core-ethereum'

const log = Debug('hopr-core:message:packet')
const verbose = Debug('hopr-core:verbose:message:packet')

/**
 * Validate newly created tickets
 * @param ops
 */
export function validateCreatedTicket(myBalance: BN, ticket: Ticket) {
  if (myBalance.lt(ticket.amount.toBN())) {
    throw Error(
      `Payment channel does not have enough funds ${myBalance.toString()} < ${ticket.amount.toBN().toString()}`
    )
  }
}

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  id: PeerId,
  nodeTicketAmount: string,
  nodeTicketWinProb: number,
  senderPeerId: PeerId,
  ticket: Ticket,
  channel: Channel,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  // self
  const selfPubKey = new PublicKey(id.pubKey.marshal())
  const selfAddress = await selfPubKey.toAddress()
  // sender
  const senderB58 = senderPeerId.toB58String()
  const senderPubKey = new PublicKey(senderPeerId.pubKey.marshal())
  const ticketAmount = ticket.amount.toBN()
  const ticketCounter = ticket.epoch.toBN()
  const ticketWinProb = getWinProbabilityAsFloat(ticket.winProb)

  let channelState
  try {
    channelState = await channel.getState()
  } catch (err) {
    throw Error(`Error while validating unacknowledged ticket, state not found: '${err.message}'`)
  }

  // ticket signer MUST be the sender
  if (!ticket.getSigner().eq(senderPubKey)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket MUST have at least X amount
  if (ticketAmount.lt(new BN(nodeTicketAmount))) {
    throw Error(`Ticket amount '${ticketAmount.toString()}' is lower than '${nodeTicketAmount}'`)
  }

  // ticket MUST have at least X winning probability
  if (ticketWinProb < nodeTicketWinProb) {
    throw Error(`Ticket winning probability '${ticketWinProb}' is lower than '${nodeTicketWinProb}'`)
  }

  // channel MUST be open or pending to close
  if (channelState.status === 'CLOSED') {
    throw Error(`Payment channel with '${senderB58}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our account nonce
  // (performance) we are making a request to blockchain
  const channelTicketEpoch = (await channel.getState()).ticketEpochFor(selfAddress).toBN()
  if (!ticketCounter.eq(channelTicketEpoch)) {
    throw Error(
      `Ticket epoch '${ticketCounter.toString()}' does not match our account counter ${channelTicketEpoch.toString()}`
    )
  }

  // ticket's channelIteration MUST match the current channelIteration
  const currentChannelIteration = channelState.channelEpoch
  const ticketChannelIteration = ticket.channelIteration.toBN()
  if (!ticketChannelIteration.eq(currentChannelIteration.toBN())) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticketChannelIteration.toString()} != ${currentChannelIteration.toString()}`
    )
  }

  // channel MUST have enough funds
  // (performance) we are making a request to blockchain
  const senderBalance = (await channel.getBalances()).counterparty
  if (senderBalance.toBN().lt(ticket.amount.toBN())) {
    throw Error(`Payment channel does not have enough funds`)
  }

  // channel MUST have enough funds
  // (performance) tickets are stored by key, we can't query sender's tickets efficiently
  // we retrieve all signed tickets and filter the ones between sender and target
  let signedTickets = (await getTickets()).filter(
    (signedTicket) =>
      signedTicket.counterparty.eq(selfAddress) &&
      signedTicket.epoch.toBN().eq(channelTicketEpoch) &&
      ticket.channelIteration.toBN().eq(currentChannelIteration.toBN())
  )

  // calculate total unredeemed balance
  const unredeemedBalance = signedTickets.reduce((total, signedTicket) => {
    return new BN(total.add(signedTicket.amount.toBN()))
  }, new BN(0))

  // ensure sender has enough funds
  if (unredeemedBalance.add(ticket.amount.toBN()).gt(senderBalance.toBN())) {
    throw Error(`Payment channel does not have enough funds when you include unredeemed tickets`)
  }
}

/**
 * Encapsulates the internal representation of a packet
 */
export class Packet extends Uint8Array {
  private _targetPeerId?: PeerId

  private _header?: Header
  private _ticket?: Ticket
  private _challenge?: Challenge
  private _message?: Message

  private libp2p: LibP2P
  private paymentChannels: HoprCoreEthereum
  private db: LevelUp
  private id: PeerId
  private ticketAmount: string
  private ticketWinProb: number

  constructor(
    libp2p: LibP2P,
    paymentChannels: HoprCoreEthereum,
    db: LevelUp,
    id: PeerId,
    ticketAmount: string,
    ticketWinProb: number,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      header: Header
      ticket: Ticket
      challenge: Challenge
      message: Message
    }
  ) {
    if (arr == null) {
      super(Packet.SIZE())
    } else {
      super(arr.bytes, arr.offset, Packet.SIZE())
    }

    if (struct != null) {
      this.set(struct.header, this.headerOffset - this.byteOffset)
      this.set(struct.ticket.serialize(), this.ticketOffset - this.byteOffset)
      this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      this.set(struct.message, this.messageOffset - this.byteOffset)

      this._header = struct.header
      this._ticket = struct.ticket
      this._challenge = struct.challenge
      this._message = struct.message
    }
    this.libp2p = libp2p
    this.paymentChannels = paymentChannels
    this.db = db
    this.id = id
    this.ticketAmount = ticketAmount
    this.ticketWinProb = ticketWinProb
  }

  slice(begin: number = 0, end: number = Packet.SIZE()) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = Packet.SIZE()): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get headerOffset(): number {
    return this.byteOffset
  }

  get header(): Header {
    if (this._header == null) {
      this._header = new Header({ bytes: this.buffer, offset: this.headerOffset })
    }

    return this._header
  }

  get ticketOffset(): number {
    return this.byteOffset + Header.SIZE
  }

  get ticket(): Promise<Ticket> {
    if (this._ticket != null) {
      return Promise.resolve(this._ticket)
    }

    return new Promise<Ticket>(async (resolve) => {
      this._ticket = await Ticket.deserialize(new Uint8Array(this.buffer, this.ticketOffset, Ticket.SIZE))
      resolve(this._ticket)
    })
  }

  get challengeOffset() {
    return this.byteOffset + Header.SIZE + Ticket.SIZE
  }

  get challenge(): Challenge {
    if (this._challenge == null) {
      this._challenge = new Challenge({
        bytes: this.buffer,
        offset: this.challengeOffset
      })
    }

    return this._challenge
  }

  get messageOffset(): number {
    return this.byteOffset + Header.SIZE + Ticket.SIZE + Challenge.SIZE()
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

  static SIZE() {
    return Header.SIZE + Ticket.SIZE + Challenge.SIZE() + Message.SIZE
  }

  /**
   * Creates a new packet.
   *
   * @param node the node itself
   * @param msg the message that is sent through the network
   * @param path array of peerId that determines the route that
   * the packet takes
   */
  static async create(node: Hopr, libp2p: LibP2P, msg: Uint8Array, path: PeerId[]): Promise<Packet> {
    const chain = node.paymentChannels
    const arr = new Uint8Array(Packet.SIZE()).fill(0x00)
    const packet = new Packet(
      libp2p,
      node.paymentChannels,
      node.db,
      node.getId(),
      node.ticketAmount,
      node.ticketWinProb,
      {
        bytes: arr.buffer,
        offset: arr.byteOffset
      }
    )

    const { header, secrets } = await Header.create(path, {
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

    packet._challenge = await Challenge.create(Hash.create(deriveTicketKeyBlinding(secrets[0])), fee, {
      bytes: packet.buffer,
      offset: packet.challengeOffset
    }).sign(libp2p.peerId)

    packet._message = Message.create(msg, {
      bytes: packet.buffer,
      offset: packet.messageOffset
    }).onionEncrypt(secrets)

    const ticketChallenge = Hash.create(
      secrets.length == 1
        ? deriveTicketLastKey(secrets[0])
        : Hash.create(
            u8aConcat(deriveTicketKey(secrets[0]), Hash.create(deriveTicketKeyBlinding(secrets[1])).serialize())
          ).serialize()
    )

    const senderPubKey = new PublicKey(node.getId().pubKey.marshal())
    const targetPubKey = new PublicKey(path[0].pubKey.marshal())
    const channel = chain.getChannel(senderPubKey, targetPubKey)

    if (secrets.length > 1) {
      log(`before creating channel`)

      const channelState = await channel.getBalances()
      packet._ticket = await channel.createTicket(new Balance(fee), ticketChallenge, node.ticketWinProb)
      validateCreatedTicket(channelState.self.toBN(), packet._ticket)
    } else if (secrets.length == 1) {
      packet._ticket = await channel.createDummyTicket(ticketChallenge)
    }

    return packet
  }

  /**
   * Checks the packet and transforms it such that it can be send to the next node.
   *
   * @param node the node itself
   */
  async forwardTransform(): Promise<{
    receivedChallenge: Challenge
    ticketKey: Uint8Array
  }> {
    const ethereum = this.paymentChannels
    this.header.deriveSecret(this.libp2p.peerId.privKey.marshal())

    if (await this.testAndSetTag(this.db)) {
      verbose('Error setting tag')
      throw Error('Error setting tag')
    }

    if (!this.header.verify()) {
      verbose('Error verifying header', this.header)
      throw Error('Error verifying header')
    }

    this.header.extractHeaderInformation()

    let isRecipient = u8aEquals(this.libp2p.peerId.pubKey.marshal(), this.header.address)

    let sender: PeerId, target: PeerId
    if (!isRecipient) {
      ;[sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

      const senderPubKey = new PublicKey(sender.pubKey.marshal())
      const targetPubKey = new PublicKey(target.pubKey.marshal())
      const channel = ethereum.getChannel(senderPubKey, targetPubKey)

      try {
        await validateUnacknowledgedTicket(
          this.id,
          this.ticketAmount,
          this.ticketWinProb,
          sender,
          await this.ticket,
          channel,
          () =>
            getTickets(this.db, {
              signer: sender.pubKey.marshal()
            })
        )
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
    if (!Hash.create(deriveTicketLastKey(this.header.derivedSecret)).eq((await this.ticket).challenge as Hash)) {
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
    const chain = this.paymentChannels
    const ticket = await this.ticket
    const sender = this.id
    const senderPubKey = new PublicKey(sender.pubKey.marshal())
    const targetPubKey = new PublicKey(target.pubKey.marshal())
    const challenge = u8aConcat(deriveTicketKey(this.header.derivedSecret), this.header.hashedKeyHalf)

    if (!Hash.create(challenge).hash().eq(ticket.challenge)) {
      verbose('Error preparing to forward')
      throw Error('Error preparing forward')
    }

    const unacknowledged = new UnacknowledgedTicket(ticket, new Hash(deriveTicketKey(this.header.derivedSecret)))

    log(
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${green(
        u8aToHex(this.header.hashedKeyHalf)
      )} from ${blue(target.toB58String())}`
    )
    await this.db.put(
      Buffer.from(UnAcknowledgedTickets(this.header.hashedKeyHalf)),
      Buffer.from(unacknowledged.serialize())
    )

    // get new ticket amount
    const fee = new Balance(ticket.amount.toBN().isub(new BN(this.ticketAmount)))
    const channel = chain.getChannel(senderPubKey, targetPubKey)

    if (fee.toBN().gtn(0)) {
      const balances = await channel.getBalances()
      this._ticket = await channel.createTicket(fee, new Hash(this.header.encryptionKey), this.ticketWinProb)
      validateCreatedTicket(balances.self.toBN(), this._ticket)
    } else if (fee.toBN().isZero()) {
      this._ticket = await channel.createDummyTicket(new Hash(this.header.encryptionKey))
    } else {
      throw Error(`Cannot forward packet`)
    }

    this.header.transformForNextNode()

    this._challenge = await Challenge.create(new Hash(this.header.hashedKeyHalf), fee.toBN(), {
      bytes: this.buffer,
      offset: this.challengeOffset
    }).sign(sender)
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
    return await pubKeyToPeerId((await this.ticket).getSigner().serialize())
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
