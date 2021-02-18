import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel } from '@hoprnet/hopr-core-connector-interface'
import type PeerId from 'peer-id'
import type Hopr from '..'
import { u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { UnacknowledgedTicket } from '../messages/ticket/unacknowledged'

type OperationSuccess = { status: 'SUCCESS'; receipt: string }
type OperationFailure = { status: 'FAILURE'; message: string }
type OperationError = { status: 'ERROR'; error: Error | string }
export type OperationStatus = OperationSuccess | OperationFailure | OperationError

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
        if (filter?.signer && !u8aEquals(await (await unAckTicket.signedTicket).signer, filter.signer)) {
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
          key: Buffer.from(node._dbKeys.UnAcknowledgedTickets((await ticket.signedTicket).ticket.challenge))
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
        const index = node._dbKeys.AcknowledgedTicketsParse(key)
        const ackTicket = await node.paymentChannels.types.AcknowledgedTicket.deserialize(value)

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !u8aEquals(await ackTicket.getSignedTicket().signer, filter.signer)) {
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
          key: Buffer.from(node._dbKeys.AcknowledgedTickets(ticket.ackTicket.getSignedTicket().ticket.challenge))
        }
      })
    )
  )
}

/**
 * Delete acknowledged ticket in database
 * @param index Uint8Array
 */
export async function deleteAcknowledgedTicket(node: Hopr<Chain>, index: Uint8Array): Promise<void> {
  await node.db.del(Buffer.from(node._dbKeys.AcknowledgedTickets(index)))
}

/**
 * Redeem acknowledged ticket and update database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function redeemTicket(
  node: Hopr<Chain>,
  ackTicket: Types.AcknowledgedTicket,
  index: Uint8Array
): Promise<OperationStatus> {
  try {
    const result = await node.paymentChannels.channel.redeemTicket(ackTicket)
    await deleteAcknowledgedTicket(node, index)
    if (result.status === 'SUCCESS') {
      // TODO Store stats.
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
): Promise<Types.SignedTicket[]> {
  return Promise.all([getUnacknowledgedTickets(node, filter), getAcknowledgedTickets(node, filter)]).then(
    async ([unAcks, acks]) => {
      const unAckTickets = await Promise.all(unAcks.map((o) => o.signedTicket))
      const ackTickets = acks.map((o) => o.ackTicket.getSignedTicket())

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
export async function validateUnacknowledgedTicket({
  node,
  senderPeerId,
  signedTicket,
  getTickets
}: {
  node: Hopr<Chain>
  senderPeerId: PeerId
  signedTicket: Types.SignedTicket
  getTickets: () => Promise<Types.SignedTicket[]>
}): Promise<void> {
  const ticket = signedTicket.ticket
  const chain = node.paymentChannels
  const selfPubKey = node.getId().pubKey.marshal()
  const selfAccountId = await chain.utils.pubKeyToAccountId(selfPubKey)
  const senderB58 = senderPeerId.toB58String()
  const senderPubKey = senderPeerId.pubKey.marshal()
  const senderAccountId = await chain.utils.pubKeyToAccountId(senderPubKey)
  const amPartyA = chain.utils.isPartyA(selfAccountId, senderAccountId)

  // ticket signer MUST be the sender
  if (!u8aEquals(await signedTicket.signer, senderPubKey)) {
    throw Error(`The signer of the ticket does not match the sender`)
  }

  // ticket MUST have at least X amount
  if (ticket.amount.lt(new BN(String(node.ticketAmount)))) {
    throw Error(`Ticket amount '${ticket.amount.toString()}' is lower than '${node.ticketAmount}'`)
  }

  // ticket MUST have at least X winning probability
  const winProb = chain.utils.getWinProbabilityAsFloat(ticket.winProb)
  if (new BN(winProb).lt(new BN(String(node.ticketWinProb)))) {
    throw Error(`Ticket winning probability '${winProb}' is lower than '${node.ticketWinProb}'`)
  }

  // channel MUST be open
  // (performance) we are making a request to blockchain
  const channelIsOpen = await chain.channel.isOpen(senderPubKey)
  if (!channelIsOpen) {
    throw Error(`Payment channel with '${senderB58}' is not open`)
  }

  // channel MUST exist in our DB
  // (performance) we are making a request to blockchain
  let channel: Channel
  try {
    channel = await chain.channel.create(
      senderPubKey,
      async () => new chain.types.Public(senderPubKey),
      undefined,
      undefined
    )
  } catch (err) {
    throw Error(`Stored payment channel with '${senderB58}' not found`)
  }

  // ticket's epoch MUST match our account nonce
  // (performance) we are making a request to blockchain
  const epoch = await chain.account.ticketEpoch
  if (!ticket.epoch.eq(epoch)) {
    throw Error(`Ticket epoch '${ticket.epoch.toString()}' does not match our account counter ${epoch.toString()}`)
  }

  // ticket's channelIteration MUST match the current channelIteration
  // (performance) we are making a request to blockchain
  const currentChannelIteration = chain.utils.stateCounterToIteration((await channel.stateCounter).toNumber())
  const ticketChannelIteration = ticket.channelIteration.toNumber()
  if (ticketChannelIteration != currentChannelIteration) {
    throw Error(
      `Ticket was created for a different channel iteration ${ticketChannelIteration} != ${currentChannelIteration}`
    )
  }

  // channel MUST have enough funds
  // (performance) we are making a request to blockchain
  const senderBalance = await (amPartyA ? channel.balance_b : channel.balance_a)
  if (senderBalance.lt(ticket.amount)) {
    throw Error(`Payment channel does not have enough funds`)
  }

  // channel MUST have enough funds
  // (performance) tickets are stored by key, we can't query sender's tickets efficiently
  // we retrieve all signed tickets and filter the ones between sender and target
  const signedTickets = await getTickets().then(async (signedTickets) => {
    return signedTickets.filter((signedTicket) => {
      return (
        u8aEquals(signedTicket.ticket.counterparty, selfAccountId) &&
        signedTicket.ticket.epoch.eq(epoch) &&
        ticket.channelIteration.toNumber() === currentChannelIteration
      )
    })
  })

  // calculate total unredeemed balance
  const unredeemedBalance = signedTickets.reduce((total, signedTicket) => {
    return new BN(total.add(signedTicket.ticket.amount))
  }, new BN(0))

  // ensure sender has enough funds
  if (unredeemedBalance.add(new BN(ticket.amount)).gt(senderBalance)) {
    throw Error(`Payment channel does not have enough funds when you include unredeemed tickets`)
  }
}

/**
 * Validate newly created tickets
 * @param ops
 */
export async function validateCreatedTicket({
  myBalance,
  signedTicket
}: {
  myBalance: BN
  signedTicket: Types.SignedTicket
}) {
  const { ticket } = signedTicket

  if (myBalance.lt(ticket.amount)) {
    throw Error(`Payment channel does not have enough funds ${myBalance.toString()} < ${ticket.amount.toString()}`)
  }
}
