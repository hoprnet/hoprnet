import type { HoprdPersistentDatabase } from '@hoprnet/hopr-utils'
import type { Operation } from 'express-openapi'
import { STATUS_CODES } from '../../utils.js'
import { Hopr } from '@hoprnet/hopr-utils'


const GET: Operation = [
  async (req, res, _next) => {
    const { node }: { node: Hopr } = req.context

    try {
      const ticket_price = await node.getTicketPrice()
      return res.status(200).send(ticket_price)
    } catch (err) {
      return res
        .status(422)
        .send({ status: STATUS_CODES.UNKNOWN_FAILURE, error: err instanceof Error ? err.message : 'Unknown error' })
    }
  }
]

export default { GET }
