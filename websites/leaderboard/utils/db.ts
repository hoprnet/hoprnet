import debug from 'debug'
import fetch from 'isomorphic-fetch'
import { HOPR_DATABASE_URL, EnvironmentProps } from './env'

const log = debug('hopr-leaderboard:firebase')
const error = debug('hopr-leaderboard:firebase:error')

export interface FirebaseStateRecords {
  address: string
  available: string
  locked: string
  connected: any[]
  connectedNodes: any[]
  hoprChannelContract: string
  hoprCoverbotAddress: string
  env: EnvironmentProps
  refreshed: string
}

export interface FirebaseScoreMap {
  [address: string]: number
}

export enum FirebaseNetworkSchema {
  'basodino' = 'basodino',
}

export enum FirebaseNetworkTables {
  'score' = 'score',
  'state' = 'state',
}

export type FirebaseResponse = FirebaseScoreMap | FirebaseStateRecords

class FirebaseDatabase {
  databaseUrl: string

  constructor() {
    this.databaseUrl = `https://${HOPR_DATABASE_URL}.firebaseio.com/`
  }

  private async resolveResponse(response: void | Response) {
    if (response) {
      const json: FirebaseResponse = await response
        .json()
        .catch((err) => error(`- resolveResponse | json :: Error parsing data from response`, err))
      log(`- resolveResponse | Retrieved json ${JSON.stringify(json)}`)
      return { data: json, status: 200 }
    } else {
      error(`- resolveResponse | Failed to retrieve data.`)
      return { data: null, status: 500 }
    }
  }

  public async getSchema(schema: FirebaseNetworkSchema) {
    try {
      log(`- getSchema | Retrieving schema ${schema} from ${this.databaseUrl}`)
      const response = await fetch(`${this.databaseUrl}${schema}.json`).catch((err) =>
        error(`- getSchema | fetch :: Error retrieve data from database`, err),
      )
      return this.resolveResponse(response)
    } catch (err) {
      error(`- getSchema | catch :: Error retrieving data`, err)
      return { data: null, status: 500 }
    }
  }

  public async getTable(schema: FirebaseNetworkSchema, table: FirebaseNetworkTables) {
    try {
      log(`- getTable | Retrieving table ${table} within schema ${schema} from ${this.databaseUrl}`)
      const response = await fetch(`${this.databaseUrl}${schema}/${table}.json`)
      return this.resolveResponse(response)
    } catch (err) {
      error(`- getTable | catch :: Error retrieving data`, err)
      return { data: null, status: 500 }
    }
  }
}

export default new FirebaseDatabase()
