import type { NextApiResponse, NextApiRequest } from 'next'

export default function handler(_req: NextApiRequest, res: NextApiResponse) {
  res.status(200).json({ status: 'ok' })
}
