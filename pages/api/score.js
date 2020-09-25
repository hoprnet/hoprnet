import { getScore } from '../../utils/api'

export default async (req, res) => {
  res.statusCode = 200
  res.json(await getScore())
}
