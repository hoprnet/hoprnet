import db from '../../utils/db'
import { HOPR_ENVIRONMENT } from '../../utils/env'

export async function get() {
  try {
    const snapshot = await db.ref(`/${HOPR_ENVIRONMENT}/state`).once('value')
    const data = snapshot.val()
    return data
  } catch (e) {
    console.log(e)
    return {}
  }
}

export default async (req, res) => {
  res.statusCode = 200
  res.json(get())
}
