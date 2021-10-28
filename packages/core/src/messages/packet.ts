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
  ChannelStatus,
  Balance,
  PRICE_PER_PACKET,
  INVERSE_TICKET_WIN_PROB
} from '@hoprnet/hopr-utils'
import type { HalfKey, HalfKeyChallenge, ChannelEntry, Challenge, Hash } from '@hoprnet/hopr-utils'
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
  privKey: PeerId
): Promise<Ticket> {
  const channel = await db.getChannelTo(dest)
  const currentTicketIndex = await bumpTicketIndex(channel.getId(), db)
  const amount = new Balance(PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB).muln(pathLength - 1))
  const winProb = new BN(INVERSE_TICKET_WIN_PROB)

  const ticket = Ticket.create(
    dest.toAddress(),
    challenge,
    channel.ticketEpoch,
    currentTicketIndex,
    amount,
    UINT256.fromInverseProbability(winProb),
    channel.channelEpoch,
    privKey.privKey.marshal()
  )
  await db.markPending(ticket)

  log(`Creating ticket in channel ${channel.getId().toHex()}. Ticket data: \n${ticket.toString()}`)

  return ticket
}

/**
 * Creates a ticket without any value
 * @param dest recipient of the ticket
 * @param challenge challenge to solve
 * @param privKey private key of the sender
 * @returns a ticket
 */
export function createZeroHopTicket(dest: PublicKey, challenge: Challenge, privKey: PeerId): Ticket {
  return Ticket.create(
    dest.toAddress(),
    challenge,
    UINT256.fromString('0'),
    UINT256.fromString('0'),
    new Balance(new BN(0)),
    UINT256.DUMMY_INVERSE_PROBABILITY,
    UINT256.fromString('0'),
    privKey.privKey.marshal()
  )
}

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

// Precompute the base unit that is used for issuing and validating
// the embedded value in tickets.
// Having this as a constant allows to channel rounding error when
// dealing with probabilities != 1.0 and makes sure that ticket value
// are always an integer multiple of the base unit.

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  id: PeerId,
  nodeTicketAmount: BN,
  nodeInverseTicketWinProb: BN,
  senderPeerId: PeerId,
  ticket: Ticket,
  channelState: ChannelEntry,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  // self
  const selfPubKey = new PublicKey(id.pubKey.marshal())
  const selfAddress = selfPubKey.toAddress()
  // sender
  const senderB58 = senderPeerId.toB58String()
  const senderPubKey = new PublicKey(senderPeerId.pubKey.marshal())
  const ticketAmount = ticket.amount.toBN()
  const ticketEpoch = ticket.epoch.toBN()
  const ticketIndex = ticket.index.toBN()
  const ticketWinProb = ticket.winProb.toBN()

  // ticket signer MUST be the sender
  if (!ticket.verify(senderPubKey)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  if (UINT256.DUMMY_INVERSE_PROBABILITY.toBN().eq(ticketWinProb) && ticketAmount.eqn(0)) {
    // Dummy ticket detected, ticket has no value and is therefore valid
    return
  }

  // ticket MUST have at least X amount
  if (ticketAmount.lt(nodeTicketAmount)) {
    throw Error(`Ticket amount '${ticketAmount.toString()}' is lower than '${nodeTicketAmount}'`)
  }

  // ticket MUST have at least X winning probability
  if (ticketWinProb.lt(UINT256.fromInverseProbability(nodeInverseTicketWinProb).toBN())) {
    throw Error(`Ticket winning probability '${ticketWinProb}' is lower than '${nodeInverseTicketWinProb}'`)
  }

  // channel MUST be open or pending to close
  if (channelState.status === ChannelStatus.Closed) {
    throw Error(`Payment channel with '${senderB58}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our account nonce
  const channelTicketEpoch = channelState.ticketEpoch.toBN()
  if (!ticketEpoch.eq(channelTicketEpoch)) {
    throw Error(
      `Ticket epoch '${ticketEpoch.toString()}' does not match our account epoch ${channelTicketEpoch.toString()}`
    )
  }

  // ticket's index MUST be higher than our account nonce
  // TODO: keep track of uncommited tickets
  const channelTicketIndex = channelState.ticketIndex.toBN()
  if (!ticketIndex.gt(channelTicketIndex)) {
    throw Error(
      `Ticket index '${ticketIndex.toString()}' must be higher than last ticket index ${channelTicketIndex.toString()}`
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
  const senderBalance = channelState.balance
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
      ticket = createZeroHopTicket(nextPeer, ticketChallenge, privKey)
    } else {
      ticket = await createTicket(nextPeer, path.length, ticketChallenge, db, privKey)
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

    await db.storePendingAcknowledgement(this.ackChallenge, true, unacknowledged)
  }

  async storePendingAcknowledgement(db: HoprDB) {
    await db.storePendingAcknowledgement(this.ackChallenge, false)
  }

  async validateUnacknowledgedTicket(db: HoprDB, privKey: PeerId) {
    const previousHop = this.previousHop.toPeerId()
    const channel = await db.getChannelFrom(this.previousHop)

    return validateUnacknowledgedTicket(
      privKey,
      PRICE_PER_PACKET,
      INVERSE_TICKET_WIN_PROB,
      previousHop,
      this.ticket,
      channel,
      () =>
        db.getTickets({
          signer: this.previousHop
        })
    )
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
      this.ticket = createZeroHopTicket(nextPeer, this.nextChallenge, privKey)
    } else {
      this.ticket = await createTicket(nextPeer, pathPosition, this.nextChallenge, db, privKey)
    }
    this.oldChallenge = this.challenge.clone()
    this.challenge = AcknowledgementChallenge.create(this.ackChallenge, privKey)

    this.isReadyToForward = true
  }
}
