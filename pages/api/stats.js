import { getStats } from '../../utils/api'

export default async (req, res) => {
  res.statusCode = 200
  res.json(await getStats())
}
