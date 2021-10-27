import {
  Ticket,
  UINT256,
  PublicKey,
  UnacknowledgedTicket,
  HoprDB,
  getPacketLength,
  POR_STRING_LENGTH,
  deriveAckKeyShare,
  createPacket,
  forwardTransform,
  generateKeyShares,
  createPoRString,
  createPoRValuesForSender,
  preVerify,
  u8aSplit,
  pubKeyToPeerId,
  HalfKeyChallenge,
  HalfKey,
  Challenge,
  ChannelEntry,
  ChannelStatus,
  Balance,
  Hash,
  PRICE_PER_PACKET,
  INVERSE_TICKET_WIN_PROB
} from '@hoprnet/hopr-utils'
import { AcknowledgementChallenge } from './acknowledgementChallenge'
import type PeerId from 'peer-id'
import BN from 'bn.js'
import { Acknowledgement } from './acknowledgement'
import { blue, green } from 'chalk'
import { debug } from '@hoprnet/hopr-utils'

export const INTERMEDIATE_HOPS = 3 // 3 relayers and 1 destination

const PACKET_LENGTH = getPacketLength(INTERMEDIATE_HOPS + 1, POR_STRING_LENGTH, 0)

const log = debug('hopr-core:message:packet')

async function bumpTicketIndex(channelId: Hash, db: HoprDB): Promise<UINT256> {
  let currentTicketIndex = await db.getCurrentTicketIndex(channelId)

  if (currentTicketIndex == undefined) {
    currentTicketIndex = new UINT256(new BN(1))
  }

  await db.setCurrentTicketIndex(channelId, new UINT256(currentTicketIndex.toBN().addn(1)))

  return currentTicketIndex
}

/**
 * Creates a signed ticket that includes the given amount of
 * tokens
 * @dev Due to a missing feature, namely ECMUL, in Ethereum, the
 * challenge is given as an Ethereum address because the signature
 * recovery algorithm is used to perform an EC-point multiplication.
 * @param amount value of the ticket
 * @param challenge challenge to solve in order to redeem the ticket
 * @param winProb the winning probability to use
 * @returns a signed ticket
 */
export async function createTicket(
  dest: PublicKey,
  pathLength: number,
  challenge: Challenge,
  db: HoprDB,
  privKey: Uint8Array
) {
  const channel = await db.getChannelTo(dest)
  const currentTicketIndex = await bumpTicketIndex(channel.getId(), db)
  const amount = new Balance(PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB).muln(pathLength - 1))
  const winProb = new BN(INVERSE_TICKET_WIN_PROB)

  /*
   * As we issue probabilistic tickets, we can't be sure of the exact balance
   * of our channels, but we can see the bounds based on how many tickets are
   * outstanding.
   */
  const outstandingTicketBalance = await db.getPendingBalanceTo(dest.toAddress())
  const balance = channel.balance.toBN().sub(outstandingTicketBalance.toBN())
  if (balance.lt(amount.toBN())) {
    throw Error(
      `We don't have enough funds in channel ${channel
        .getId()
        .toHex()} with counterparty ${dest.toB58String()} to create ticket`
    )
  }

  const ticket = Ticket.create(
    dest.toAddress(),
    challenge,
    channel.ticketEpoch,
    currentTicketIndex,
    amount,
    UINT256.fromInverseProbability(winProb),
    channel.channelEpoch,
    privKey
  )
  await db.markPending(ticket)

  log(`Creating ticket in channel ${channel.getId().toHex()}. Ticket data: \n${ticket.toString()}`)

  return ticket
}

// Precompute the base unit that is used for issuing and validating
// the embedded value in tickets.
// Having this as a constant allows to channel rounding error when
// dealing with probabilities != 1.0 and makes sure that ticket value
// are always an integer multiple of the base unit.

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  usPeerId: PeerId,
  themPeerId: PeerId,
  minTicketAmount: BN,
  reqInverseTicketWinProb: BN,
  ticket: Ticket,
  channel: ChannelEntry,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  const us = new PublicKey(usPeerId.pubKey.marshal())
  const them = new PublicKey(themPeerId.pubKey.marshal())
  const requiredTicketWinProb = UINT256.fromInverseProbability(reqInverseTicketWinProb).toBN()

  // ticket signer MUST be the sender
  if (!ticket.verify(them)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket amount MUST be greater or equal to minTicketAmount
  if (!ticket.amount.toBN().gte(minTicketAmount)) {
    throw Error(`Ticket amount '${ticket.amount.toBN().toString()}' is not equal to '${minTicketAmount.toString()}'`)
  }

  // ticket amount MUST be greater or equal to minTicketAmount
  if (!ticket.amount.toBN().gte(minTicketAmount)) {
    throw Error(`Ticket amount '${ticket.amount.toBN().toString()}' is not equal to '${minTicketAmount.toString()}'`)
  }

  // ticket MUST have match X winning probability
  if (!ticket.winProb.toBN().eq(requiredTicketWinProb)) {
    throw Error(
      `Ticket winning probability '${ticket.winProb
        .toBN()
        .toString()}' is not equal to '${requiredTicketWinProb.toString()}'`
    )
  }

  // channel MUST be open or pending to close
  if (channel.status === ChannelStatus.Closed) {
    throw Error(`Payment channel with '${them.toB58String()}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our account nonce
  if (!ticket.epoch.toBN().eq(channel.ticketEpoch.toBN())) {
    throw Error(
      `Ticket epoch '${ticket.epoch.toBN().toString()}' does not match our account epoch ${channel.ticketEpoch
        .toBN()
        .toString()}`
    )
  }

  // ticket's index MUST be higher than the channel's ticket index
  if (!ticket.index.toBN().gt(channel.ticketIndex.toBN())) {
    throw Error(
      `Ticket index '${ticket.index.toBN().toString()}' must be higher than last ticket index ${channel.ticketIndex
        .toBN()
        .toString()}`
    )
  }

  // ticket's channelIteration MUST match the current channelIteration
  if (!ticket.channelIteration.toBN().eq(channel.channelEpoch.toBN())) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticket.channelIteration
        .toBN()
        .toString()} != ${channel.channelEpoch.toBN().toString()}`
    )
  }

  // channel MUST have enough funds
  if (channel.balance.toBN().lt(ticket.amount.toBN())) {
    throw Error(`Payment channel does not have enough funds`)
  }

  // channel MUST have enough funds
  // (performance) tickets are stored by key, we can't query sender's tickets efficiently
  // we retrieve all signed tickets and filter the ones between sender and target
  let signedTickets = (await getTickets()).filter(
    (signedTicket) =>
      signedTicket.counterparty.eq(us.toAddress()) &&
      signedTicket.epoch.toBN().eq(channel.ticketEpoch.toBN()) &&
      ticket.channelIteration.toBN().eq(channel.channelEpoch.toBN())
  )

  // calculate total unredeemed balance
  const unredeemedBalance = signedTickets.reduce((total, signedTicket) => {
    return new BN(total.add(signedTicket.amount.toBN()))
  }, new BN(0))

  // ensure sender has enough funds
  if (unredeemedBalance.add(ticket.amount.toBN()).gt(channel.balance.toBN())) {
    throw Error(`Payment channel does not have enough funds when you include unredeemed tickets`)
  }
}

export class Packet {
  public isReceiver: boolean
  public isReadyToForward: boolean

  public plaintext: Uint8Array

  public packetTag: Uint8Array
  public previousHop: PublicKey
  public nextHop: Uint8Array
  public ownShare: HalfKeyChallenge
  public ownKey: HalfKey
  public ackKey: HalfKey
  public nextChallenge: Challenge
  public ackChallenge: HalfKeyChallenge
  public oldChallenge: AcknowledgementChallenge

  public constructor(private packet: Uint8Array, private challenge: AcknowledgementChallenge, public ticket: Ticket) {}

  private setReadyToForward(ackChallenge: HalfKeyChallenge) {
    this.ackChallenge = ackChallenge
    this.isReadyToForward = true

    return this
  }

  private setFinal(plaintext: Uint8Array, packetTag: Uint8Array, ackKey: HalfKey, previousHop: PublicKey) {
    this.packetTag = packetTag
    this.ackKey = ackKey
    this.isReceiver = true
    this.isReadyToForward = false
    this.plaintext = plaintext
    this.previousHop = previousHop

    return this
  }

  private setForward(
    ackKey: HalfKey,
    ownKey: HalfKey,
    ownShare: HalfKeyChallenge,
    nextHop: Uint8Array,
    previousHop: PublicKey,
    nextChallenge: Challenge,
    ackChallenge: HalfKeyChallenge,
    packetTag: Uint8Array
  ) {
    this.isReceiver = false
    this.isReadyToForward = false

    this.ackKey = ackKey
    this.ownKey = ownKey
    this.ownShare = ownShare
    this.previousHop = previousHop
    this.nextHop = nextHop
    this.nextChallenge = nextChallenge
    this.ackChallenge = ackChallenge
    this.packetTag = packetTag

    return this
  }

  static async create(msg: Uint8Array, path: PeerId[], privKey: PeerId, db: HoprDB): Promise<Packet> {
    const isDirectMessage = path.length == 1
    const { alpha, secrets } = generateKeyShares(path)
    const { ackChallenge, ticketChallenge } = createPoRValuesForSender(secrets[0], secrets[1])
    const porStrings: Uint8Array[] = []

    for (let i = 0; i < path.length - 1; i++) {
      porStrings.push(createPoRString(secrets[i + 1], i + 2 < path.length ? secrets[i + 2] : undefined))
    }

    const challenge = AcknowledgementChallenge.create(ackChallenge, privKey)
    const nextPeer = new PublicKey(path[0].pubKey.marshal())
    const packet = createPacket(secrets, alpha, msg, path, INTERMEDIATE_HOPS + 1, POR_STRING_LENGTH, porStrings)

    let ticket: Ticket
    if (isDirectMessage) {
      // Dummy Ticket
      ticket = Ticket.create(
        nextPeer.toAddress(),
        ticketChallenge,
        UINT256.fromString('0'),
        UINT256.fromString('0'),
        new Balance(new BN(0)),
        UINT256.DUMMY_INVERSE_PROBABILITY,
        UINT256.fromString('0'),
        privKey.privKey.marshal()
      )
    } else {
      ticket = await createTicket(nextPeer, path.length, ticketChallenge, db, privKey.privKey.marshal())
    }

    return new Packet(packet, challenge, ticket).setReadyToForward(ackChallenge)
  }

  serialize(): Uint8Array {
    return Uint8Array.from([...this.packet, ...this.challenge.serialize(), ...this.ticket.serialize()])
  }

  static get SIZE() {
    return PACKET_LENGTH + AcknowledgementChallenge.SIZE + Ticket.SIZE
  }

  static deserialize(preArray: Uint8Array, privKey: PeerId, pubKeySender: PeerId): Packet {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (preArray.length != Packet.SIZE) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = Uint8Array.from(arr)
    } else {
      arr = preArray
    }

    const [packet, preChallenge, preTicket] = u8aSplit(arr, [PACKET_LENGTH, AcknowledgementChallenge.SIZE, Ticket.SIZE])

    const transformedOutput = forwardTransform(privKey, packet, POR_STRING_LENGTH, 0, INTERMEDIATE_HOPS + 1)

    const ackKey = deriveAckKeyShare(transformedOutput.derivedSecret)

    const challenge = AcknowledgementChallenge.deserialize(preChallenge, ackKey.toChallenge(), pubKeySender)

    const ticket = Ticket.deserialize(preTicket)

    if (transformedOutput.lastNode == true) {
      return new Packet(packet, challenge, ticket).setFinal(
        transformedOutput.plaintext,
        transformedOutput.packetTag,
        ackKey,
        PublicKey.fromPeerId(pubKeySender)
      )
    }

    const verificationOutput = preVerify(
      transformedOutput.derivedSecret,
      transformedOutput.additionalRelayData,
      ticket.challenge
    )

    if (verificationOutput.valid != true) {
      throw Error(`PoR value pre-verification failed.`)
    }

    return new Packet(transformedOutput.packet, challenge, ticket).setForward(
      ackKey,
      verificationOutput.ownKey,
      verificationOutput.ownShare,
      transformedOutput.nextHop,
      PublicKey.fromPeerId(pubKeySender),
      verificationOutput.nextTicketChallenge,
      verificationOutput.ackChallenge,
      transformedOutput.packetTag
    )
  }

  async checkPacketTag(db: HoprDB) {
    const present = await db.checkAndSetPacketTag(this.packetTag)

    if (present) {
      throw Error(`Potential replay attack detected. Packet tag is already present.`)
    }
  }

  async storeUnacknowledgedTicket(db: HoprDB) {
    if (this.ownKey == undefined) {
      throw Error(`Invalid state`)
    }

    const unacknowledged = new UnacknowledgedTicket(this.ticket, this.ownKey, this.previousHop)

    log(
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${green(
        this.ackChallenge.toHex()
      )} from ${blue(pubKeyToPeerId(this.nextHop).toB58String())}`
    )

    await db.storeUnacknowledgedTicket(this.ackChallenge, unacknowledged)
  }

  async validateUnacknowledgedTicket(db: HoprDB, privKey: PeerId) {
    const channel = await db.getChannelFrom(this.previousHop)

    try {
      await validateUnacknowledgedTicket(
        privKey,
        this.previousHop.toPeerId(),
        PRICE_PER_PACKET,
        INVERSE_TICKET_WIN_PROB,
        this.ticket,
        channel,
        () =>
          db.getTickets({
            signer: this.previousHop
          })
      )
    } catch (e) {
      await db.markRejected(this.ticket)
      throw e
    }

    await db.setCurrentTicketIndex(channel.getId().hash(), this.ticket.index)
  }

  createAcknowledgement(privKey: PeerId) {
    if (this.ackKey == undefined) {
      throw Error(`Invalid state`)
    }

    return Acknowledgement.create(this.oldChallenge ?? this.challenge, this.ackKey, privKey)
  }

  async forwardTransform(privKey: PeerId, db: HoprDB): Promise<void> {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (this.isReceiver || this.isReadyToForward) {
      throw Error(`Invalid state`)
    }

    const nextPeer = new PublicKey(this.nextHop)

    const pathPosition = this.ticket.getPathPosition()
    if (pathPosition == 1) {
      // Dummy Ticket
      this.ticket = Ticket.create(
        nextPeer.toAddress(),
        this.nextChallenge,
        UINT256.fromString('0'),
        UINT256.fromString('0'),
        new Balance(new BN(0)),
        UINT256.DUMMY_INVERSE_PROBABILITY,
        UINT256.fromString('0'),
        privKey.privKey.marshal()
      )
    } else {
      this.ticket = await createTicket(nextPeer, pathPosition, this.nextChallenge, db, privKey.privKey.marshal())
    }
    this.oldChallenge = this.challenge.clone()
    this.challenge = AcknowledgementChallenge.create(this.ackChallenge, privKey)

    this.isReadyToForward = true
  }
}
