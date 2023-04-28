import {
  HoprDB,
  PRICE_PER_PACKET,
  INVERSE_TICKET_WIN_PROB,
  create_counter, UINT256,
  PublicKey as TsPublicKey,
  HalfKeyChallenge as TsHalfKeyChallenge,
  Address as TsAddress,
  Balance as TsBalance,
  Ticket as TsTicket,
  UnacknowledgedTicket as TsUnacknowledgedTicket
} from '@hoprnet/hopr-utils'
import type { Hash } from '@hoprnet/hopr-utils'
import type { PeerId } from '@libp2p/interface-peer-id'
import { debug } from '@hoprnet/hopr-utils'
import { keysPBM } from '@libp2p/crypto/keys'

import { Ticket, Balance, BalanceType, ChannelStatus, ChannelEntry, PublicKey, Packet, U256, UnacknowledgedTicket, WasmPacketState } from '../../lib/core_packet.js'
export { Packet, WasmPacketState as PacketState } from '../../lib/core_packet.js'

import { peerIdFromString } from '@libp2p/peer-id'
import BN from 'bn.js'

const log = debug('hopr-core:message:packet')

// Metrics
const metric_ticketCounter = create_counter('core_counter_created_tickets', 'Number of created tickets')
const metric_packetCounter = create_counter('core_counter_packets', 'Number of created packets')

async function bumpTicketIndex(channelId: Hash, db: HoprDB): Promise<U256> {
  let fetchedTicketIndex = await db.getCurrentTicketIndex(channelId)
  let currentTicketIndex: U256

  if (fetchedTicketIndex == undefined) {
    currentTicketIndex = U256.one();
    fetchedTicketIndex = new UINT256(new BN(1))
  }

  await db.setCurrentTicketIndex(channelId, new UINT256(fetchedTicketIndex.toBN().addn(1)))

  return currentTicketIndex
}

/**
 * Creates a signed ticket that includes the given amount of
 * tokens
 * @dev Due to a missing feature, namely ECMUL, in Ethereum, the
 * challenge is given as an Ethereum address because the signature
 * recovery algorithm is used to perform an EC-point multiplication.
 * @returns a signed ticket
 * @param dest
 * @param pathLength
 * @param db
 * @param privKey
 */
async function createTicket(
  dest: PublicKey,
  pathLength: number,
  db: HoprDB,
  privKey: Uint8Array
): Promise<Ticket> {

  let ts_pk = TsPublicKey.deserialize(dest.serialize(false))

  const channel = await db.getChannelTo(ts_pk)
  const currentTicketIndex = await bumpTicketIndex(channel.getId(), db)
  const amount = new Balance(PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB).muln(pathLength - 1).toString(), BalanceType.HOPR)
  const winProb = new U256(INVERSE_TICKET_WIN_PROB.toString(10))

  /*
   * As we issue probabilistic tickets, we can't be sure of the exact balance
   * of our channels, but we can see the bounds based on how many tickets are
   * outstanding.
   */
  let ts_address = TsAddress.deserialize(dest.to_address().serialize())
  const outstandingTicketBalance = await db.getPendingBalanceTo(ts_address)
  const balance_bn = channel.balance.toBN().sub(outstandingTicketBalance.toBN())
  const balance = new Balance(balance_bn.toString(10), BalanceType.HOPR)

  log(
    `balances ${channel.balance.toFormattedString()} - ${outstandingTicketBalance.toFormattedString()} = ${new TsBalance(
      balance_bn
    ).toFormattedString()} should >= ${amount.to_string()} in channel open to ${
      !channel.destination ? '' : channel.destination.toString()
    }`
  )
  if (balance.lt(amount)) {
    throw Error(
      `We don't have enough funds in channel ${channel
        .getId()
        .toHex()} with counterparty ${dest.toString()} to create ticket`
    )
  }

  const ticket = Ticket.new(dest.to_address(), undefined,
    new U256(channel.ticketEpoch.toBN().toString(10)),
    currentTicketIndex,
    amount,
    U256.from_inverse_probability(winProb),
    new U256(channel.channelEpoch.toBN().toString(10)),
    privKey)

  await db.markPending(TsTicket.deserialize(ticket.serialize()))

  log(`Creating ticket in channel ${channel.getId().toHex()}. Ticket data: \n${ticket.to_hex()}`)
  metric_ticketCounter.increment()

  return ticket
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
  minTicketAmount: Balance,
  reqInverseTicketWinProb: U256,
  ticket: Ticket,
  channel: ChannelEntry,
  getTickets: () => Promise<Ticket[]>,
  checkUnrealizedBalance: boolean
): Promise<void> {
  const them = PublicKey.from_peerid_str(themPeerId.toString())
  const requiredTicketWinProb = U256.from_inverse_probability(reqInverseTicketWinProb)

  // ticket signer MUST be the sender
  if (!ticket.verify(them)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket amount MUST be greater or equal to minTicketAmount
  if (!ticket.amount.gte(minTicketAmount)) {
    throw Error(`Ticket amount '${ticket.amount.to_string()}' is not equal to '${minTicketAmount.to_string()}'`)
  }

  // ticket MUST have match X winning probability
  if (!ticket.win_prob.eq(requiredTicketWinProb)) {
    throw Error(
      `Ticket winning probability '${ticket.win_prob.to_string()}' is not equal to '${requiredTicketWinProb.to_string()}'`
    )
  }

  // channel MUST be open or pending to close
  if (channel.status === ChannelStatus.Closed) {
    throw Error(`Payment channel with '${them.toString()}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our channel's epoch
  if (!ticket.epoch.eq(channel.ticket_epoch)) {
    throw Error(
      `Ticket epoch '${ticket.epoch.to_string()}' does not match our account epoch ${channel.ticket_epoch
        .to_string()} of channel ${channel.get_id().to_hex()}`
    )
  }

  // ticket's channelEpoch MUST match the current channel's epoch
  if (!ticket.channel_epoch.eq(channel.channel_epoch)) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticket.channel_epoch
        .to_string()} != ${channel.channel_epoch} of channel ${channel.get_id().to_hex()}`
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

export function privateKeyFromPeer(peer: PeerId) {
  if (peer.privateKey == undefined)
    throw Error('peer id does not contain a private key')

  return keysPBM.PrivateKey.decode(peer.privateKey).Data
}


/**
 * This is a temporary helper class until the DB functionality is migrated to Rust.
 */
export class PacketHelper {
  static async create(msg: Uint8Array, path: PeerId[], privKey: PeerId, db: HoprDB): Promise<Packet> {

    let private_key = privateKeyFromPeer(privKey)

    let next_peer = PublicKey.from_peerid_str(path[0].toString())

    let ticket: Ticket;
    if (path.length == 1) {
      ticket = Ticket.new_zero_hop(next_peer, undefined, private_key)
    } else {
        ticket = await createTicket(next_peer, path.length, db, private_key)
    }

    metric_packetCounter.increment()

    return new Packet(msg, path.map((p) => p.toString()), private_key, ticket)
  }

  static async checkPacketTag(packet: Packet, db: HoprDB) {
    const present = await db.checkAndSetPacketTag(packet.packet_tag())

    if (present) {
      throw Error(`Potential replay attack detected. Packet tag is already present.`)
    }
  }

  static async storeUnacknowledgedTicket(packet: Packet, db: HoprDB) {
    if (packet.state() != WasmPacketState.Forwarded) {
      throw Error(`Invalid state`)
    }

    const unacknowledged = new UnacknowledgedTicket(packet.ticket, packet.own_key(), packet.previous_hop())

    log(
      `Storing unacknowledged ticket. Expecting to receive a preImage for ${packet.ack_challenge().to_hex()} from ${
        packet.next_hop().to_peerid_str()
      }`
    )

    await db.storePendingAcknowledgement(TsHalfKeyChallenge.deserialize(packet.ack_challenge().serialize()),
      false,
      TsUnacknowledgedTicket.deserialize(unacknowledged.serialize())
      )
  }

  static async storePendingAcknowledgement(packet: Packet, db: HoprDB) {
    let hkc = TsHalfKeyChallenge.deserialize(packet.ack_challenge().serialize())
    await db.storePendingAcknowledgement(hkc, true)
  }

  static async validateUnacknowledgedTicket(packet: Packet, db: HoprDB, checkUnrealizedBalance: boolean) {
    if (packet.state() == WasmPacketState.Outgoing) {
      throw Error('packet must have previous hop - cannot be outgoing')
    }

    const channel = await db.getChannelFrom(TsPublicKey.deserialize(packet.previous_hop().serialize(false)))

    try {
      await validateUnacknowledgedTicket(
        peerIdFromString(packet.previous_hop().to_peerid_str()),
        new Balance(PRICE_PER_PACKET.toString(), BalanceType.HOPR),
        new U256(INVERSE_TICKET_WIN_PROB.toString()),
        packet.ticket,
        ChannelEntry.deserialize(channel.serialize()),
        async () => {
          let prev_hop = TsPublicKey.deserialize(packet.previous_hop().serialize(false))
          let tickets = await db.getTickets({
            signer: prev_hop
          })
          return tickets.map((ts_ticket) => Ticket.deserialize(ts_ticket.serialize()))
        },
        checkUnrealizedBalance
      )
    } catch (e) {
      log(`mark ticket as rejected`, packet.ticket.to_hex())
      await db.markRejected(TsTicket.deserialize(packet.ticket.serialize()))
      throw e
    }

    await db.setCurrentTicketIndex(channel.getId().hash(), new UINT256(new BN(packet.ticket.index.to_string())))
  }

  static async forwardTransform(packet: Packet, privKey: PeerId, db: HoprDB): Promise<void> {
    if (privKey.privateKey == null) {
      throw Error(`Invalid arguments`)
    }

    let private_key = privateKeyFromPeer(privKey)
    const pathPosition = packet.ticket.get_path_position(new U256(PRICE_PER_PACKET.toString()), new U256(INVERSE_TICKET_WIN_PROB.toString()))

    let nextPeer = packet.next_hop()

    let ticket: Ticket
    if (pathPosition == 1) {
      ticket = Ticket.new_zero_hop(nextPeer, undefined, private_key)
    } else {
      ticket = await createTicket(nextPeer, pathPosition, db, private_key)
    }

    packet.forward(private_key, ticket)
  }
}
