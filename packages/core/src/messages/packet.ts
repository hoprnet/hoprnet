import {
  Ticket,
  U256,
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
  BalanceType,
  PRICE_PER_PACKET,
  INVERSE_TICKET_WIN_PROB,
  create_counter
} from '@hoprnet/hopr-utils'
import type { HalfKey, HalfKeyChallenge, ChannelEntry, Challenge, Hash } from '@hoprnet/hopr-utils'
import { AcknowledgementChallenge, Acknowledgement } from '../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import BN from 'bn.js'
import { debug } from '@hoprnet/hopr-utils'
import { keysPBM } from '@libp2p/crypto/keys'
import { peerIdFromString } from '@libp2p/peer-id'

export const INTERMEDIATE_HOPS = 3 // 3 relayers and 1 destination

const PACKET_LENGTH = getPacketLength(INTERMEDIATE_HOPS + 1, POR_STRING_LENGTH, 0)

const log = debug('hopr-core:message:packet')

// Metrics
const metric_ticketCounter = create_counter('core_counter_created_tickets', 'Number of created tickets')
const metric_packetCounter = create_counter('core_counter_packets', 'Number of created packets')

async function bumpTicketIndex(channelId: Hash, db: HoprDB): Promise<U256> {
  let currentTicketIndex = await db.getCurrentTicketIndex(channelId)

  if (currentTicketIndex == undefined) {
    currentTicketIndex = U256.one()
  }

  await db.setCurrentTicketIndex(channelId, currentTicketIndex.addn(1))

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
  if (!privKey.privateKey) {
    throw Error(`Cannot create acknowledgement because lacking access to private key`)
  }

  const channel = await db.getChannelTo(dest)
  const currentTicketIndex = await bumpTicketIndex(channel.get_id(), db)
  const amount = new Balance(PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB).muln(pathLength - 1).toString(10), BalanceType.HOPR)
  const winProb = new BN(INVERSE_TICKET_WIN_PROB)

  /*
   * As we issue probabilistic tickets, we can't be sure of the exact balance
   * of our channels, but we can see the bounds based on how many tickets are
   * outstanding.
   */
  const outstandingTicketBalance = await db.getPendingBalanceTo(dest.to_address())
  const balance = channel.balance.sub(outstandingTicketBalance)
  log(
    `balances ${channel.balance.to_formatted_string()} - ${outstandingTicketBalance.to_formatted_string()} = ${
      balance.to_formatted_string()} should >= ${amount.to_formatted_string()} in channel open to ${
      !channel.destination ? '' : channel.destination.toString()
    }`
  )
  if (balance.lt(amount)) {
    throw Error(
      `We don't have enough funds in channel ${channel
        .get_id().to_hex()} with counterparty ${dest.toString()} to create ticket`
    )
  }

  const ticket = Ticket.new(
    dest.to_address(),
    challenge,
    channel.ticket_epoch,
    currentTicketIndex,
    amount,
    U256.from_inverse_probability(new U256(winProb.toString())),
    channel.channel_epoch,
    keysPBM.PrivateKey.decode(privKey.privateKey).Data
  )
  await db.markPending(ticket)

  log(`Creating ticket in channel ${channel.get_id().to_hex()}. Ticket data: \n${ticket.toString()}`)
  metric_ticketCounter.increment()

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
  if (!privKey.privateKey) {
    throw Error(`Cannot create acknowledgement because lacking access to private key`)
  }

  return Ticket.new(
    dest.to_address(),
    challenge,
    U256.zero(),
    U256.zero(),
    Balance.zero(BalanceType.HOPR),
    U256.zero(),
    U256.zero(),
    keysPBM.PrivateKey.decode(privKey.privateKey).Data
  )
}

// Precompute the base unit that is used for issuing and validating
// the embedded value in tickets.
// Having this as a constant allows to channel rounding error when
// dealing with probabilities != 1.0 and makes sure that ticket value
// are always an integer multiple of the base unit.

/**
 * Validate unacknowledged tickets as we receive them.
 * Out of order validation is allowed. Ordering is enforced
 * when tickets are redeemed.
 */
export async function validateUnacknowledgedTicket(
  themPeerId: PeerId,
  minTicketAmount: BN,
  reqInverseTicketWinProb: BN,
  ticket: Ticket,
  channel: ChannelEntry,
  getTickets: () => Promise<Ticket[]>,
  checkUnrealizedBalance: boolean
): Promise<void> {
  const them = PublicKey.from_peerid_str(themPeerId.toString())
  const requiredTicketWinProb = U256.from_inverse_probability(new U256(reqInverseTicketWinProb.toString()))

  // ticket signer MUST be the sender
  if (!ticket.verify(them)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket amount MUST be greater or equal to minTicketAmount
  if (!ticket.amount.gte(new Balance(minTicketAmount.toString(10), BalanceType.HOPR))) {
    throw Error(`Ticket amount '${ticket.amount.to_string()}' is not equal to '${minTicketAmount.toString()}'`)
  }

  // ticket MUST have match X winning probability
  if (!ticket.win_prob.eq(requiredTicketWinProb)) {
    throw Error(
      `Ticket winning probability '${ticket.win_prob.to_string()}' is not equal to '${requiredTicketWinProb.toString()}'`
    )
  }

  // channel MUST be open or pending to close
  if (channel.status === ChannelStatus.Closed) {
    throw Error(`Payment channel with '${them.toString()}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our channel's epoch
  if (!ticket.epoch.eq(channel.ticket_epoch)) {
    throw Error(
      `Ticket epoch '${ticket.epoch.to_string()}' does not match our account epoch ${channel.ticket_epoch.to_string()}
        of channel ${channel.get_id().to_hex()}`
    )
  }

  // ticket's channelEpoch MUST match the current channel's epoch
  if (!ticket.channel_epoch.eq(channel.channel_epoch)) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticket.channel_epoch}
         != ${channel.channel_epoch} of channel ${channel.get_id().to_hex()}`
    )
  }

  if (checkUnrealizedBalance) {
    // find out pending balance from unredeemed tickets
    log(`checking unrealized balances for channel ${channel.get_id().to_hex()}`)

    // all tickets from sender
    const tickets = await getTickets().then((ts) => {
      return ts.filter((t) => {
        return t.epoch.eq(channel.ticket_epoch) && t.channel_epoch.eq(channel.channel_epoch)
      })
    })

    const unrealizedBalance = tickets.reduce((result, t) => {
      // update balance
      result = result.sub(t.amount)

      return result
    }, channel.balance)

    // ensure sender has enough funds
    if (ticket.amount.gt(unrealizedBalance)) {
      throw Error(`Payment channel ${channel.get_id().to_hex()} does not have enough funds`)
    }
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

  public constructor(private packet: Uint8Array, private challenge: AcknowledgementChallenge, public ticket: Ticket) {
    metric_packetCounter.increment()
  }

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

    const challenge = new AcknowledgementChallenge(ackChallenge, keysPBM.PrivateKey.decode(privKey.privateKey).Data)
    const nextPeer = PublicKey.from_peerid_str(path[0].toString())
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
    return PACKET_LENGTH + AcknowledgementChallenge.size() + Ticket.size()
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
      arr = new Uint8Array(preArray.buffer, preArray.byteOffset, preArray.byteLength)
    } else {
      arr = preArray
    }

    const [packet, preChallenge, preTicket] = u8aSplit(arr, [PACKET_LENGTH, AcknowledgementChallenge.size(), Ticket.size()])

    const transformedOutput = forwardTransform(privKey, packet, POR_STRING_LENGTH, 0, INTERMEDIATE_HOPS + 1)

    const ackKey = deriveAckKeyShare(transformedOutput.derivedSecret)

    const challenge = AcknowledgementChallenge.deserialize(preChallenge)
    challenge.validate(ackKey.to_challenge(), PublicKey.from_peerid_str(pubKeySender.toString()))

    const ticket = Ticket.deserialize(preTicket)

    if (transformedOutput.lastNode == true) {
      return new Packet(packet, challenge, ticket).setFinal(
        transformedOutput.plaintext,
        transformedOutput.packetTag,
        ackKey,
        PublicKey.from_peerid_str(pubKeySender.toString())
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
      PublicKey.from_peerid_str(pubKeySender.toString()),
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
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${this.ackChallenge.to_hex()} from ${pubKeyToPeerId(
        this.nextHop
      ).toString()}`
    )

    await db.storePendingAcknowledgement(this.ackChallenge, false, unacknowledged)
  }

  async storePendingAcknowledgement(db: HoprDB) {
    await db.storePendingAcknowledgement(this.ackChallenge, true)
  }

  async validateUnacknowledgedTicket(db: HoprDB, checkUnrealizedBalance: boolean) {
    const channel = await db.getChannelFrom(this.previousHop)

    try {
      await validateUnacknowledgedTicket(
        peerIdFromString(this.previousHop.to_peerid_str()),
        PRICE_PER_PACKET,
        INVERSE_TICKET_WIN_PROB,
        this.ticket,
        channel,
        () =>
          db.getTickets({
            signer: this.previousHop
          }),
        checkUnrealizedBalance
      )
    } catch (e) {
      log(`mark ticket as rejected`, this.ticket.toString())
      await db.markRejected(this.ticket)
      throw e
    }

    await db.setCurrentTicketIndex(channel.get_id(), this.ticket.index)
  }

  createAcknowledgement(privKey: PeerId) {
    if (this.ackKey == undefined) {
      throw Error(`Invalid state`)
    }
    let pk = keysPBM.PrivateKey.decode(privKey.privateKey).Data
    return new Acknowledgement(this.oldChallenge ?? this.challenge, this.ackKey, pk)
  }

  async forwardTransform(privKey: PeerId, db: HoprDB): Promise<void> {
    if (privKey.privateKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (this.isReceiver || this.isReadyToForward) {
      throw Error(`Invalid state`)
    }

    const nextPeer = PublicKey.deserialize(this.nextHop)

    const pathPosition = this.ticket.get_path_position(new U256(PRICE_PER_PACKET.toString(10)), new U256(INVERSE_TICKET_WIN_PROB.toString(10)))
    if (pathPosition == 1) {
      this.ticket = createZeroHopTicket(nextPeer, this.nextChallenge, privKey)
    } else {
      this.ticket = await createTicket(nextPeer, pathPosition, this.nextChallenge, db, privKey)
    }
    this.oldChallenge = this.challenge.clone()
    let pk = keysPBM.PrivateKey.decode(privKey.privateKey).Data
    this.challenge = new AcknowledgementChallenge(this.ackChallenge, pk)

    this.isReadyToForward = true
  }
}
