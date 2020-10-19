import db, { FirebaseResponse, FirebaseNetworkSchema, FirebaseNetworkTables } from './db'
import { HOPR_NETWORK } from './env'

export type APIFirebaseResponse = {
  data: FirebaseResponse
  status: number
}

export type APIResponseVoid = {
  data: null
  status: 500
}

export type APIResponse = APIFirebaseResponse | APIResponseVoid

export async function getData(table: FirebaseNetworkTables): Promise<APIResponse> {
  try {
    const queryResponse = await db.getTable(HOPR_NETWORK as FirebaseNetworkSchema, table)
    if (queryResponse.data) {
      return { data: queryResponse.data, status: 200 }
    } else {
      return { data: null, status: 500 }
    }
  } catch (e) {
    console.log(e)
    return { data: null, status: 500 }
  }
}

export async function getState() {
  return getData(FirebaseNetworkTables.state)
}

export async function getScore() {
  return getData(FirebaseNetworkTables.score)
}

export default { getState, getScore }
