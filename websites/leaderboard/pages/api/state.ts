import { getState } from '../../utils/api'
import { NextApiRequest, NextApiResponse } from 'next'
import { FirebaseStateRecords } from '../../utils/db'

export default async (_req: NextApiRequest, res: NextApiResponse): Promise<FirebaseStateRecords | void> => {
  res.statusCode = 200
  const response = await getState()
  if (response.data) {
    const data = response.data as FirebaseStateRecords
    return res.json(data)
  } else {
    return res.json({})
  }
}
