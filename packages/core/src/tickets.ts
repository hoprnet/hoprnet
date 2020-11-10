import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '.'
import { UnacknowledgedTicket } from './messages/ticket/unacknowledged'

type OperationSuccess = { status: 'SUCCESS'; receipt: string }
type OperationFailure = { status: 'FAILURE'; message: string }
type OperationError = { status: 'ERROR'; error: Error | string }
export type OperationStatus = OperationSuccess | OperationFailure | OperationError

class Tickets<Chain extends HoprCoreConnector> {
  constructor(private node: Hopr<Chain>) {}

  /**
   * Get all unacknowledged tickets
   * @returns an array of all unacknowledged tickets
   */
  public async getUnacknowledgedTickets(): Promise<UnacknowledgedTicket<Chain>[]> {
    const tickets: UnacknowledgedTicket<Chain>[] = []

    return new Promise((resolve, reject) => {
      this.node.db
        .createReadStream({
          gte: Buffer.from(this.node._dbKeys.UnAcknowledgedTickets(new Uint8Array(0x00)))
        })
        .on('error', (err) => reject(err))
        .on('data', ({ value }: { value: Buffer }) => {
          tickets.push(
            new UnacknowledgedTicket(this.node.paymentChannels, {
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
  public async getAcknowledgedTickets(): Promise<
    {
      ackTicket: Types.AcknowledgedTicket
      index: Uint8Array
    }[]
  > {
    const { AcknowledgedTicket } = this.node.paymentChannels.types
    const acknowledgedTicketSize = AcknowledgedTicket.SIZE(this.node.paymentChannels)
    let promises: {
      ackTicket: Types.AcknowledgedTicket
      index: Uint8Array
    }[] = []

    return new Promise((resolve, reject) => {
      this.node.db
        .createReadStream({
          gte: Buffer.from(this.node._dbKeys.AcknowledgedTickets(new Uint8Array(0x00)))
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          if (value.buffer.byteLength !== acknowledgedTicketSize) return

          const index = this.node._dbKeys.AcknowledgedTicketsParse(key)
          const ackTicket = AcknowledgedTicket.create(this.node.paymentChannels, {
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
   * Update Acknowledged Ticket in database
   * @param ackTicket Uint8Array
   * @param index Uint8Array
   */
  public async updateAcknowledgedTicket(ackTicket: Types.AcknowledgedTicket, index: Uint8Array): Promise<void> {
    await this.node.db.put(Buffer.from(this.node._dbKeys.AcknowledgedTickets(index)), Buffer.from(ackTicket))
  }

  /**
   * Delete Acknowledged Ticket in database
   * @param index Uint8Array
   */
  public async deleteAcknowledgedTicket(index: Uint8Array): Promise<void> {
    await this.node.db.del(Buffer.from(this.node._dbKeys.AcknowledgedTickets(index)))
  }

  /**
   * Submit Acknowledged Ticket and update database
   * @param ackTicket Uint8Array
   * @param index Uint8Array
   */
  public async submitAcknowledgedTicket(
    ackTicket: Types.AcknowledgedTicket,
    index: Uint8Array
  ): Promise<OperationStatus> {
    try {
      const result = await this.node.paymentChannels.channel.tickets.submit(ackTicket, index)

      if (result.status === 'SUCCESS') {
        ackTicket.redeemed = true
        await this.updateAcknowledgedTicket(ackTicket, index)
      } else if (result.status === 'FAILURE') {
        await this.deleteAcknowledgedTicket(index)
      } else if (result.status === 'ERROR') {
        await this.deleteAcknowledgedTicket(index)
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
}

export default Tickets
