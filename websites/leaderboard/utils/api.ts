import db from './db'
import { HOPR_ENVIRONMENT } from './env'

export async function getData({ table }: { table: string }) {
  try {
    const snapshot = await db.ref(`/${HOPR_ENVIRONMENT}/${table}`).once('value')
    const data = snapshot.val()
    return data || {}
  } catch (e) {
    console.log(e)
    return {}
  }
}

export async function getStats() {
  return getData({ table: 'state' })
}

export async function getScore() {
  return getData({ table: 'score' })
}

export default { getStats, getScore }
