import type PeerId from 'peer-id'
import type Hopr from '..'
import { u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { UnacknowledgedTicket } from '../messages/ticket/unacknowledged'
import {
  PublicKey,
  Ticket,
  Channel,
  Acknowledgement,
  SubmitTicketResponse,
  getWinProbabilityAsFloat
} from '@hoprnet/hopr-core-ethereum'

/**
 * Get all unacknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all unacknowledged tickets
 */
export async function getUnacknowledgedTickets(
  node: Hopr,
  filter?: {
    signer: Uint8Array
  }
): Promise<UnacknowledgedTicket[]> {
  const tickets: UnacknowledgedTicket[] = []

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.UnAcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', async ({ value }: { value: Buffer }) => {
        if (value.buffer.byteLength !== UnacknowledgedTicket.SIZE()) return

        const unAckTicket = new UnacknowledgedTicket({
          bytes: value.buffer,
          offset: value.byteOffset
        })

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals((await unAckTicket.signedTicket).getSigner().serialize(), filter.signer)) {
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
  node: Hopr,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const tickets = await getUnacknowledgedTickets(node, filter)

  await node.db.batch(
    await Promise.all(
      tickets.map<any>(async (ticket) => {
        return {
          type: 'del',
          key: Buffer.from(node._dbKeys.UnAcknowledgedTickets((await ticket.signedTicket).challenge.serialize()))
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
  node: Hopr,
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
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.AcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
        if (value.buffer.byteLength !== Acknowledgement.SIZE) return

        const index = node._dbKeys.AcknowledgedTicketsParse(key)
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
  node: Hopr,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const acks = await getAcknowledgements(node, filter)
  await node.db.batch(
    await Promise.all(
      acks.map<any>(async (ack) => {
        return {
          type: 'del',
          key: Buffer.from(node._dbKeys.AcknowledgedTickets((await ack.ackTicket.ticket).challenge.serialize()))
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
export async function updateAcknowledgement(node: Hopr, ackTicket: Acknowledgement, index: Uint8Array): Promise<void> {
  await node.db.put(Buffer.from(node._dbKeys.AcknowledgedTickets(index)), Buffer.from(ackTicket.serialize()))
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
  node: Hopr,
  filter?: {
    signer: Uint8Array
  }
): Promise<Ticket[]> {
  return Promise.all([getUnacknowledgedTickets(node, filter), getAcknowledgements(node, filter)]).then(
    async ([unAcks, acks]) => {
      const unAckTickets = await Promise.all(unAcks.map((o) => o.signedTicket))
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
  node: Hopr,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  await Promise.all([deleteUnacknowledgedTickets(node, filter), deleteAcknowledgements(node, filter)])
}

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  node: Hopr,
  senderPeerId: PeerId,
  ticket: Ticket,
  channel: Channel,
  getTickets: () => Promise<Ticket[]>
): Promise<void> {
  const ethereum = node.paymentChannels
  // self
  const selfPubKey = new PublicKey(node.getId().pubKey.marshal())
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
  if (ticketAmount.lt(new BN(node.ticketAmount))) {
    throw Error(`Ticket amount '${ticketAmount.toString()}' is lower than '${node.ticketAmount}'`)
  }

  // ticket MUST have at least X winning probability
  if (ticketWinProb < node.ticketWinProb) {
    throw Error(`Ticket winning probability '${ticketWinProb}' is lower than '${node.ticketWinProb}'`)
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

/**
 * Validate newly created tickets
 * @param ops
 */
export async function validateCreatedTicket({ myBalance, ticket }: { myBalance: BN; ticket: Ticket }) {
  if (myBalance.lt(ticket.amount.toBN())) {
    throw Error(`Payment channel does not have enough funds ${myBalance.toString()} < ${ticket.amount.toString()}`)
  }
}
