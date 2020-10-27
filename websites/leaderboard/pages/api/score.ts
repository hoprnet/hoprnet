import { getScore } from '../../utils/api'
import { NextApiRequest, NextApiResponse } from 'next'
import { FirebaseScoreMap } from '../../utils/db'

export default async (_req: NextApiRequest, res: NextApiResponse): Promise<FirebaseScoreMap[] | void> => {
  res.statusCode = 200
  const response = await getScore()
  if (response.data) {
    const data = response.data as FirebaseScoreMap
    return res.json(data)
  } else {
    return res.json({})
  }
}
