import { Channel } from '@hoprnet/hopr-core-ethereum'
import {
  Ticket,
  UINT256,
  PublicKey,
  Balance,
  UnacknowledgedTicket,
  HoprDB,
  getPacketLength,
  POR_STRING_LENGTH,
  deriveAckKeyShare,
  createPacket,
  forwardTransform,
  generateKeyShares,
  createPoRString,
  createFirstChallenge,
  preVerify,
  u8aSplit,
  pubKeyToPeerId,
  HalfKeyChallenge,
  HalfKey,
  Challenge as ChallengeType
} from '@hoprnet/hopr-utils'
import type HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { Challenge } from './challenge'
import type PeerId from 'peer-id'
import BN from 'bn.js'
import { Acknowledgement } from './acknowledgement'
import { blue, green } from 'chalk'
import Debug from 'debug'

export const MAX_HOPS = 3 // 3 relayers and 1 destination

const PACKET_LENGTH = getPacketLength(MAX_HOPS + 1, POR_STRING_LENGTH, 0)

const log = Debug('hopr-core:message:packet')

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
  const selfAddress = selfPubKey.toAddress()
  // sender
  const senderB58 = senderPeerId.toB58String()
  const senderPubKey = new PublicKey(senderPeerId.pubKey.marshal())
  const ticketAmount = ticket.amount.toBN()
  const ticketEpoch = ticket.epoch.toBN()
  const ticketIndex = ticket.index.toBN()
  const ticketWinProb = ticket.winProb.toBN()

  let channelState
  try {
    channelState = await channel.getState()
  } catch (err) {
    throw Error(`Error while validating unacknowledged ticket, state not found: '${err.message}'`)
  }

  // ticket signer MUST be the sender
  if (!ticket.verify(senderPubKey)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket MUST have at least X amount
  if (ticketAmount.lt(new BN(nodeTicketAmount))) {
    throw Error(`Ticket amount '${ticketAmount.toString()}' is lower than '${nodeTicketAmount}'`)
  }

  // ticket MUST have at least X winning probability
  if (ticketWinProb.lt(UINT256.fromProbability(nodeTicketWinProb).toBN())) {
    throw Error(`Ticket winning probability '${ticketWinProb}' is lower than '${nodeTicketWinProb}'`)
  }

  // channel MUST be open or pending to close
  if (channelState.status === 'CLOSED') {
    throw Error(`Payment channel with '${senderB58}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our account nonce
  const channelTicketEpoch = (await channel.getState()).ticketEpochFor(selfAddress).toBN()
  if (!ticketEpoch.eq(channelTicketEpoch)) {
    throw Error(
      `Ticket epoch '${ticketEpoch.toString()}' does not match our account epoch ${channelTicketEpoch.toString()}`
    )
  }

  // ticket's index MUST be higher than our account nonce
  // TODO: keep track of uncommited tickets
  const channelTicketIndex = (await channel.getState()).ticketIndexFor(selfAddress).toBN()
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

export class Packet {
  public isReceiver: boolean
  public isReadyToForward: boolean

  public plaintext: Uint8Array

  public packetTag: Uint8Array
  public previousHop: Uint8Array
  public nextHop: Uint8Array
  public ownShare: HalfKeyChallenge
  public ownKey: HalfKey
  public ackKey: HalfKey
  public nextChallenge: ChallengeType
  public ackChallenge: HalfKeyChallenge

  public constructor(private packet: Uint8Array, private challenge: Challenge, private ticket: Ticket) {}

  private setReadyToForward(ackChallenge: HalfKeyChallenge) {
    this.ackChallenge = ackChallenge
    this.isReadyToForward = true

    return this
  }

  private setFinal(plaintext: Uint8Array, packetTag: Uint8Array, ackKey: HalfKey) {
    this.packetTag = packetTag
    this.ackKey = ackKey
    this.isReceiver = true
    this.isReadyToForward = false
    this.plaintext = plaintext

    return this
  }

  private setForward(
    ackKey: HalfKey,
    ownKey: HalfKey,
    ownShare: HalfKeyChallenge,
    nextHop: Uint8Array,
    previousHop: Uint8Array,
    nextChallenge: ChallengeType,
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

  static async create(
    msg: Uint8Array,
    path: PeerId[],
    privKey: PeerId,
    chain: HoprCoreEthereum,
    ticketOpts: {
      value: Balance
      winProb: number
    }
  ): Promise<Packet> {
    const isDirectMessage = path.length === 1
    const { alpha, secrets } = generateKeyShares(path)
    const { ackChallenge, ticketChallenge } = createFirstChallenge(secrets[0], secrets[1])
    const porStrings: Uint8Array[] = []

    for (let i = 0; i < path.length - 1; i++) {
      porStrings.push(createPoRString(secrets[i + 1], i + 2 < path.length ? secrets[i + 2] : undefined))
    }

    const challenge = Challenge.create(ackChallenge, privKey)
    const self = new PublicKey(privKey.pubKey.marshal())
    const nextPeer = new PublicKey(path[0].pubKey.marshal())
    const packet = createPacket(secrets, alpha, msg, path, MAX_HOPS + 1, POR_STRING_LENGTH, porStrings)
    const channel = chain.getChannel(self, nextPeer)

    let ticket: Ticket
    if (isDirectMessage) {
      ticket = channel.createDummyTicket(ticketChallenge)
    } else {
      ticket = await channel.createTicket(ticketOpts.value, ticketChallenge, ticketOpts.winProb)
    }

    return new Packet(packet, challenge, ticket).setReadyToForward(ackChallenge)
  }

  serialize(): Uint8Array {
    return Uint8Array.from([...this.packet, ...this.challenge.serialize(), ...this.ticket.serialize()])
  }

  static get SIZE() {
    return PACKET_LENGTH + Challenge.SIZE + Ticket.SIZE
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

    const [packet, preChallenge, preTicket] = u8aSplit(arr, [PACKET_LENGTH, Challenge.SIZE, Ticket.SIZE])

    const transformedOutput = forwardTransform(privKey, packet, POR_STRING_LENGTH, 0, MAX_HOPS + 1)

    const ackKey = deriveAckKeyShare(transformedOutput.derivedSecret)

    const challenge = Challenge.deserialize(preChallenge, ackKey.toChallenge(), pubKeySender)

    const ticket = Ticket.deserialize(preTicket)

    if (transformedOutput.lastNode == true) {
      return new Packet(packet, challenge, ticket).setFinal(
        transformedOutput.plaintext,
        transformedOutput.packetTag,
        ackKey
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
      pubKeySender.pubKey.marshal(),
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

    const unacknowledged = new UnacknowledgedTicket(this.ticket, this.ownKey)

    log(
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${green(
        this.ackChallenge.toHex()
      )} from ${blue(pubKeyToPeerId(this.nextHop).toB58String())}`
    )

    await db.storeUnacknowledgedTickets(this.ackChallenge, unacknowledged)
  }

  async validateUnacknowledgedTicket(db: HoprDB, chain: HoprCoreEthereum, privKey: PeerId) {
    const previousHop = pubKeyToPeerId(this.previousHop)
    const channel = chain.getChannel(new PublicKey(privKey.pubKey.marshal()), new PublicKey(this.previousHop))

    return validateUnacknowledgedTicket(privKey, '', 0, previousHop, this.ticket, channel, () =>
      db.getTickets({
        signer: this.previousHop
      })
    )
  }

  createAcknowledgement(privKey: PeerId) {
    if (this.ackKey == undefined) {
      throw Error(`Invalid state`)
    }

    return Acknowledgement.create(this.challenge, this.ackKey, privKey)
  }

  async forwardTransform(privKey: PeerId, chain: HoprCoreEthereum): Promise<void> {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (this.isReceiver || this.isReadyToForward) {
      throw Error(`Invalid state`)
    }

    const self = new PublicKey(privKey.pubKey.marshal())
    const nextPeer = new PublicKey(this.nextHop)

    const channel = chain.getChannel(self, nextPeer)

    this.ticket = await channel.createTicket(new Balance(new BN(0)), this.nextChallenge, 0)

    this.challenge = Challenge.create(this.ackChallenge, privKey)

    this.isReadyToForward = true
  }
}
