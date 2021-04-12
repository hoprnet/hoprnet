import type PeerId from 'peer-id'
import type Hopr from '..'
import { u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { LevelUp } from 'levelup'
import HoprCoreEthereum, {
  PublicKey,
  Ticket,
  Channel,
  Acknowledgement,
  SubmitTicketResponse,
  UnacknowledgedTicket,
  getWinProbabilityAsFloat
} from '@hoprnet/hopr-core-ethereum'
import { UnAcknowledgedTickets, AcknowledgedTickets, AcknowledgedTicketsParse } from '../dbKeys'

/**
 * Get all unacknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all unacknowledged tickets
 */
export async function getUnacknowledgedTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<UnacknowledgedTicket[]> {
  const tickets: UnacknowledgedTicket[] = []

  return new Promise((resolve, reject) => {
    db.createReadStream({
      gte: Buffer.from(UnAcknowledgedTickets(new Uint8Array(0x00)))
    })
      .on('error', (err: any) => reject(err))
      .on('data', async ({ value }: { value: Buffer }) => {
        if (value.buffer.byteLength !== UnacknowledgedTicket.SIZE()) return

        const unAckTicket = UnacknowledgedTicket.deserialize(value)

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals(unAckTicket.ticket.getSigner().serialize(), filter.signer)) {
          return
        }

        tickets.push(unAckTicket)
      })
      .on('end', () => resolve(tickets))
  })
}

/**
 * Delete unacknowledged tickets
 * @param filter optionally filter by signer
 */
export async function deleteUnacknowledgedTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const tickets = await getUnacknowledgedTickets(db, filter)

  await db.batch(
    await Promise.all(
      tickets.map<any>(async (ticket) => {
        return {
          type: 'del',
          key: Buffer.from(UnAcknowledgedTickets(ticket.ticket.challenge.serialize()))
        }
      })
    )
  )
}

/**
 * Get all acknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all acknowledged tickets
 */
export async function getAcknowledgements(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<
  {
    ackTicket: Acknowledgement
    index: Uint8Array
  }[]
> {
  const results: {
    ackTicket: Acknowledgement
    index: Uint8Array
  }[] = []

  return new Promise((resolve, reject) => {
    db.createReadStream({
      gte: Buffer.from(AcknowledgedTickets(new Uint8Array(0x00)))
    })
      .on('error', (err) => reject(err))
      .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
        if (value.buffer.byteLength !== Acknowledgement.SIZE) return

        const index = AcknowledgedTicketsParse(key)
        const ackTicket = Acknowledgement.deserialize(value)

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals((await ackTicket.ticket).getSigner().serialize(), filter.signer)) {
          return
        }

        results.push({
          ackTicket,
          index
        })
      })
      .on('end', () => resolve(results))
  })
}

/**
 * Delete acknowledged tickets
 * @param filter optionally filter by signer
 */
export async function deleteAcknowledgements(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const acks = await getAcknowledgements(db, filter)
  await db.batch(
    await Promise.all(
      acks.map<any>(async (ack) => {
        return {
          type: 'del',
          key: Buffer.from(AcknowledgedTickets((await ack.ackTicket.ticket).challenge.serialize()))
        }
      })
    )
  )
}

/**
 * Update acknowledged ticket in database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function updateAcknowledgement(db: LevelUp, ackTicket: Acknowledgement, index: Uint8Array): Promise<void> {
  await db.put(Buffer.from(AcknowledgedTickets(index)), Buffer.from(ackTicket.serialize()))
}

/**
 * Delete acknowledged ticket in database
 * @param index Uint8Array
 */
export async function deleteAcknowledgement(node: Hopr, index: Uint8Array): Promise<void> {
  await node.db.del(Buffer.from(node._dbKeys.AcknowledgedTickets(index)))
}

/**
 * Submit acknowledged ticket and update database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function submitAcknowledgedTicket(
  node: Hopr,
  ackTicket: Acknowledgement,
  index: Uint8Array
): Promise<SubmitTicketResponse> {
  try {
    const ethereum = node.paymentChannels
    const signedTicket = await ackTicket.ticket
    const self = ethereum.account.keys.onChain.pubKey
    const counterparty = signedTicket.getSigner()
    const channel = new ethereum.channel(ethereum, self, counterparty)

    const result = await channel.submitTicket(ackTicket)
    // TODO look at result.status and actually do something
    await deleteAcknowledgement(node, index)
    return result
  } catch (err) {
    return {
      status: 'ERROR',
      error: err
    }
  }
}

/**
 * Get signed tickets, both unacknowledged and acknowledged
 * @param node
 * @param filter optionally filter by signer
 * @returns an array of signed tickets
 */
export async function getTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<Ticket[]> {
  return Promise.all([getUnacknowledgedTickets(db, filter), getAcknowledgements(db, filter)]).then(
    async ([unAcks, acks]) => {
      const unAckTickets = await Promise.all(unAcks.map((o) => o.ticket))
      const ackTickets = await Promise.all(acks.map((o) => o.ackTicket.ticket))

      return [...unAckTickets, ...ackTickets]
    }
  )
}

/**
 * Get signed tickets, both unacknowledged and acknowledged
 * @param node
 * @param filter optionally filter by signer
 * @returns an array of signed tickets
 */
export async function deleteTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  await Promise.all([deleteUnacknowledgedTickets(db, filter), deleteAcknowledgements(db, filter)])
}

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  paymentChannels: HoprCoreEthereum,
  id: PeerId,
  nodeTicketAmount: string,
  nodeTicketWinProb: number,
  senderPeerId: PeerId,
  ticket: Ticket,
  channel: Channel,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  const ethereum = paymentChannels
  // self
  const selfPubKey = new PublicKey(id.pubKey.marshal())
  const selfAddress = await selfPubKey.toAddress()
  // sender
  const senderB58 = senderPeerId.toB58String()
  const senderPubKey = new PublicKey(senderPeerId.pubKey.marshal())
  const ticketAmount = ticket.amount.toBN()
  const ticketCounter = ticket.epoch.toBN()
  const accountCounter = (await ethereum.account.getTicketEpoch()).toBN()
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
  if (channelState.getStatus() === 'CLOSED') {
    throw Error(`Payment channel with '${senderB58}' is not open or pending to close`)
  }

  // ticket's epoch MUST match our account nonce
  // (performance) we are making a request to blockchain
  if (!ticketCounter.eq(accountCounter)) {
    throw Error(
      `Ticket epoch '${ticketCounter.toString()}' does not match our account counter ${accountCounter.toString()}`
    )
  }

  // ticket's channelIteration MUST match the current channelIteration
  const currentChannelIteration = channelState.getIteration()
  const ticketChannelIteration = ticket.channelIteration.toBN()
  if (!ticketChannelIteration.eq(currentChannelIteration)) {
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
      signedTicket.epoch.toBN().eq(accountCounter) &&
      ticket.channelIteration.toBN().eq(currentChannelIteration)
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

