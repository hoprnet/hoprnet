import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
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

  return new Promise((resolve, reject) => {
    node.db
      .createReadStream({
        gte: Buffer.from(node._dbKeys.UnAcknowledgedTickets(new Uint8Array(0x00)))
      })
      .on('error', (err) => reject(err))
      .on('data', ({ value }: { value: Buffer }) => {
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
