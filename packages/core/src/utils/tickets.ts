import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel, SubmitTicketResponse } from '@hoprnet/hopr-core-connector-interface'
import type PeerId from 'peer-id'
import type Hopr from '..'
import { u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { UnacknowledgedTicket } from '../messages/ticket/unacknowledged'
import { PublicKey } from '@hoprnet/hopr-core-ethereum'

/**
 * Get all unacknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all unacknowledged tickets
 */
export async function getUnacknowledgedTickets(
  node: Hopr<Chain>,
  filter?: {
    signer: Uint8Array
  }
): Promise<UnacknowledgedTicket<Chain>[]> {
  const tickets: UnacknowledgedTicket<Chain>[] = []
  const unAcknowledgedTicketSize = UnacknowledgedTicket.SIZE(node.paymentChannels)

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.UnAcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', async ({ value }: { value: Buffer }) => {
        if (value.buffer.byteLength !== unAcknowledgedTicketSize) return

        const unAckTicket = new UnacknowledgedTicket(node.paymentChannels, {
          bytes: value.buffer,
          offset: value.byteOffset
        })

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals((await unAckTicket.signedTicket).getSigner(), filter.signer)) {
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
  node: Hopr<Chain>,
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
export async function getAcknowledgedTickets(
  node: Hopr<Chain>,
  filter?: {
    signer: Uint8Array
  }
): Promise<
  {
    ackTicket: Types.AcknowledgedTicket
    index: Uint8Array
  }[]
> {
  const { AcknowledgedTicket } = node.paymentChannels.types
  const results: {
    ackTicket: Types.AcknowledgedTicket
    index: Uint8Array
  }[] = []

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.AcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
        if (value.buffer.byteLength !== AcknowledgedTicket.SIZE) return

        const index = node._dbKeys.AcknowledgedTicketsParse(key)
        const ackTicket = AcknowledgedTicket.create({
          bytes: value.buffer,
          offset: value.byteOffset
        })

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals(await (await ackTicket.signedTicket).signer, filter.signer)) {
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
export async function deleteAcknowledgedTickets(
  node: Hopr<Chain>,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const tickets = await getAcknowledgedTickets(node, filter)

  await node.db.batch(
    await Promise.all(
      tickets.map<any>(async (ticket) => {
        return {
          type: 'del',
          key: Buffer.from(
            node._dbKeys.AcknowledgedTickets((await ticket.ackTicket.signedTicket).ticket.challenge.serialize())
          )
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
export async function updateAcknowledgedTicket(
  node: Hopr<Chain>,
  ackTicket: Types.AcknowledgedTicket,
  index: Uint8Array
): Promise<void> {
  await node.db.put(Buffer.from(node._dbKeys.AcknowledgedTickets(index)), Buffer.from(ackTicket))
}

/**
 * Delete acknowledged ticket in database
 * @param index Uint8Array
 */
export async function deleteAcknowledgedTicket(node: Hopr<Chain>, index: Uint8Array): Promise<void> {
  await node.db.del(Buffer.from(node._dbKeys.AcknowledgedTickets(index)))
}

/**
 * Submit acknowledged ticket and update database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function submitAcknowledgedTicket(
  node: Hopr<Chain>,
  ackTicket: Types.AcknowledgedTicket,
  index: Uint8Array
): Promise<SubmitTicketResponse> {
  try {
    const ethereum = node.paymentChannels
    const signedTicket = await ackTicket.signedTicket
    const self = new PublicKey(ethereum.account.keys.onChain.pubKey)
    const counterparty = new PublicKey(await signedTicket.signer)
    const channel = new ethereum.channel(ethereum, self, counterparty)

    const result = await channel.submitTicket(ackTicket, index)
    if (result.status === 'SUCCESS') {
      ackTicket.redeemed = true
      await updateAcknowledgedTicket(node, ackTicket, index)
    } else if (result.status === 'FAILURE') {
      await deleteAcknowledgedTicket(node, index)
    } else if (result.status === 'ERROR') {
      await deleteAcknowledgedTicket(node, index)
      // @TODO: better handle this
    }

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
  node: Hopr<Chain>,
  filter?: {
    signer: Uint8Array
  }
): Promise<Types.Ticket[]> {
  return Promise.all([getUnacknowledgedTickets(node, filter), getAcknowledgedTickets(node, filter)]).then(
    async ([unAcks, acks]) => {
      const unAckTickets = await Promise.all(unAcks.map((o) => o.signedTicket))
      const ackTickets = await Promise.all(acks.map((o) => o.ackTicket.signedTicket))

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
  node: Hopr<Chain>,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  await Promise.all([deleteUnacknowledgedTickets(node, filter), deleteAcknowledgedTickets(node, filter)])
}

/**
 * Validate unacknowledged tickets as we receive them
 */
export async function validateUnacknowledgedTicket(
  node: Hopr<Chain>,
  senderPeerId: PeerId,
  ticket: Types.Ticket,
  channel: Channel,
  getTickets: () => Promise<Types.Ticket[]>
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
  const ticketWinProb = ethereum.utils.getWinProbabilityAsFloat(ticket.winProb)

  let channelState: Types.ChannelEntry
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
export async function validateCreatedTicket({ myBalance, ticket }: { myBalance: BN; ticket: Types.Ticket }) {
  if (myBalance.lt(ticket.amount.toBN())) {
    throw Error(`Payment channel does not have enough funds ${myBalance.toString()} < ${ticket.amount.toString()}`)
  }
}
