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
 * @returns an array of all unacknowledged tickets
 */
export async function getUnacknowledgedTickets(node: Hopr<Chain>): Promise<UnacknowledgedTicket<Chain>[]> {
  const tickets: UnacknowledgedTicket<Chain>[] = []
  const unAcknowledgedTicketSize = UnacknowledgedTicket.SIZE(node.paymentChannels)

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.UnAcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', ({ value }: { value: Buffer }) => {
        if (value.buffer.byteLength !== unAcknowledgedTicketSize) return

        tickets.push(
          new UnacknowledgedTicket(node.paymentChannels, {
            bytes: value.buffer,
            offset: value.byteOffset
          })
        )
      })
      .on('end', () => resolve(Promise.all(tickets)))
  })
}

/**
 * Get all acknowledged tickets
 * @returns an array of all acknowledged tickets
 */
export async function getAcknowledgedTickets(
  node: Hopr<Chain>
): Promise<
  {
    ackTicket: Types.AcknowledgedTicket
    index: Uint8Array
  }[]
> {
  const { AcknowledgedTicket } = node.paymentChannels.types
  const acknowledgedTicketSize = AcknowledgedTicket.SIZE(node.paymentChannels)
  let promises: {
    ackTicket: Types.AcknowledgedTicket
    index: Uint8Array
  }[] = []

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.AcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
        if (value.buffer.byteLength !== acknowledgedTicketSize) return

        const index = node._dbKeys.AcknowledgedTicketsParse(key)
        const ackTicket = AcknowledgedTicket.create(node.paymentChannels, {
          bytes: value.buffer,
          offset: value.byteOffset
        })

        promises.push({
          ackTicket,
          index
        })
      })
      .on('end', () => resolve(Promise.all(promises)))
  })
}

/**
 * Get all signed tickets, both unacknowledged and acknowledged
 * @param node
 * @returns an array of all signed tickets
 */
export async function getAllTickets(node: Hopr<Chain>): Promise<Types.SignedTicket[]> {
  return Promise.all([getUnacknowledgedTickets(node), getAcknowledgedTickets(node)]).then(async ([unAcks, acks]) => {
    const unAckTickets = await Promise.all(unAcks.map((o) => o.signedTicket))
    const ackTickets = await Promise.all(acks.map((o) => o.ackTicket.signedTicket))
    return [...unAckTickets, ...ackTickets]
  })
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
): Promise<OperationStatus> {
  try {
    const result = await node.paymentChannels.channel.tickets.submit(ackTicket, index)

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

// NOTE: currently validating tickets is not performant
export async function validateUnacknowledgedTicket({
  node,
  senderPeerId,
  targetPeerId,
  signedTicket,
  getAllTickets
}: {
  node: Hopr<Chain>
  senderPeerId: PeerId
  targetPeerId: PeerId
  signedTicket: Types.SignedTicket
  getAllTickets: () => Promise<Types.SignedTicket[]>
}): Promise<void> {
  const ticket = signedTicket.ticket
  const chain = node.paymentChannels
  const selfPubKey = targetPeerId.pubKey.marshal()
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

  // channel MUST have enough funds
  // (performance) we are making a request to blockchain
  const senderBalance = await (amPartyA ? channel.balance_b : channel.balance_a)
  if (senderBalance.lt(ticket.amount)) {
    throw Error(`Payment channel does not have enough funds`)
  }

  // channel MUST have enough funds
  // (performance) tickets are stored by key, we can't query sender's tickets efficiently
  // we retrieve all signed tickets and filter the ones between sender and target
  const tickets = await getAllTickets().then(async (signedTickets) => {
    const tickets: Types.Ticket[] = []
    let signedTicket: Types.SignedTicket

    while ((signedTicket = signedTickets.pop())) {
      const signer = await signedTicket.signer

      const valid =
        u8aEquals(signedTicket.ticket.counterparty, selfPubKey) &&
        u8aEquals(signer, senderPubKey) &&
        signedTicket.ticket.epoch.eq(epoch)

      if (!valid) continue

      tickets.push(signedTicket.ticket)
    }

    return tickets
  })

  // calculate total unredeemed balance
  const unredeemedBalance = tickets.reduce((total, ticket) => {
    return new BN(total.add(ticket.amount))
  }, new BN(0))

  // ensure sender has enough funds
  if (unredeemedBalance.add(new BN(ticket.amount)).gt(senderBalance)) {
    throw Error(`Payment channel does not have enough funds when you include unredeemed tickets`)
  }
}
