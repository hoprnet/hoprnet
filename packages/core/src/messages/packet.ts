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
import { AcknowledgementChallenge } from './acknowledgementChallenge.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import BN from 'bn.js'
import { Acknowledgement } from './acknowledgement.js'
import { debug } from '@hoprnet/hopr-utils'
import { keysPBM } from '@libp2p/crypto/keys'

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

  /*
   * As we issue probabilistic tickets, we can't be sure of the exact balance
   * of our channels, but we can see the bounds based on how many tickets are
   * outstanding.
   */
  const outstandingTicketBalance = await db.getPendingBalanceTo(dest.toAddress())
  const balance = channel.balance.toBN().sub(outstandingTicketBalance.toBN())
  log(
    `balances ${channel.balance.toFormattedString()} - ${outstandingTicketBalance.toFormattedString()} = ${new Balance(
      balance
    ).toFormattedString()} should >= ${amount.toFormattedString()} in channel open to ${
      !channel.destination ? '' : channel.destination.toString()
    }`
  )
  if (balance.lt(amount.toBN())) {
    throw Error(
      `We don't have enough funds in channel ${channel
        .getId()
        .toHex()} with counterparty ${dest.toString()} to create ticket`
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
    keysPBM.PrivateKey.decode(privKey.privateKey).Data
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
    keysPBM.PrivateKey.decode(privKey.privateKey).Data
  )
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
  themPeerId: PeerId,
  minTicketAmount: BN,
  reqInverseTicketWinProb: BN,
  ticket: Ticket,
  channel: ChannelEntry,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  const them = PublicKey.fromPeerId(themPeerId)
  const requiredTicketWinProb = UINT256.fromInverseProbability(reqInverseTicketWinProb).toBN()

  // ticket signer MUST be the sender
  if (!ticket.verify(them)) {
    throw Error(`The signer of the ticket does not match the sender`)
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
    throw Error(`Payment channel with '${them.toString()}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our channel's epoch
  if (!ticket.epoch.toBN().eq(channel.ticketEpoch.toBN())) {
    throw Error(
      `Ticket epoch '${ticket.epoch.toBN().toString()}' does not match our account epoch ${channel.ticketEpoch
        .toBN()
        .toString()} of channel ${channel.getId().toHex()}`
    )
  }

  // ticket's channelEpoch MUST match the current channel's epoch
  if (!ticket.channelEpoch.toBN().eq(channel.channelEpoch.toBN())) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticket.channelEpoch
        .toBN()
        .toString()} != ${channel.channelEpoch.toBN().toString()} of channel ${channel.getId().toHex()}`
    )
  }

  // find out latest index and pending balance
  // from unredeemed tickets

  // all tickets from sender
  const tickets = await getTickets().then((ts) => {
    return ts.filter((t) => {
      return t.epoch.toBN().eq(channel.ticketEpoch.toBN()) && t.channelEpoch.toBN().eq(channel.channelEpoch.toBN())
    })
  })

  const { unrealizedBalance, unrealizedIndex } = tickets.reduce(
    (result, t) => {
      // update index
      if (result.unrealizedIndex.toBN().lt(t.index.toBN())) {
        result.unrealizedIndex = t.index
      }

      // update balance
      result.unrealizedBalance = result.unrealizedBalance.sub(t.amount)

      return result
    },
    {
      unrealizedBalance: channel.balance,
      unrealizedIndex: channel.ticketIndex
    }
  )

  // ticket's index MUST be higher than the channel's ticket index
  if (ticket.index.toBN().lt(unrealizedIndex.toBN())) {
    throw Error(
      `Ticket index ${ticket.index.toBN().toString()} for channel ${channel
        .getId()
        .toHex()} must be higher than last ticket index ${unrealizedIndex.toBN().toString()}`
    )
  }

  // ensure sender has enough funds
  if (ticket.amount.toBN().gt(unrealizedBalance.toBN())) {
    throw Error(`Payment channel ${channel.getId().toHex()} does not have enough funds`)
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
    const nextPeer = PublicKey.fromPeerId(path[0])
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
    if (!privKey.privateKey) {
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
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${this.ackChallenge.toHex()} from ${pubKeyToPeerId(
        this.nextHop
      ).toString()}`
    )

    await db.storePendingAcknowledgement(this.ackChallenge, false, unacknowledged)
  }

  async storePendingAcknowledgement(db: HoprDB) {
    await db.storePendingAcknowledgement(this.ackChallenge, true)
  }

  async validateUnacknowledgedTicket(db: HoprDB) {
    const channel = await db.getChannelFrom(this.previousHop)

    try {
      await validateUnacknowledgedTicket(
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
      log(`mark ticket as rejected`, this.ticket.toString())
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
    if (privKey.privateKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (this.isReceiver || this.isReadyToForward) {
      throw Error(`Invalid state`)
    }

    const nextPeer = PublicKey.deserialize(this.nextHop)

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
