import debug from 'debug'
import fetch from 'isomorphic-fetch'


const log = debug('hopr-chatbot:firebase')
const error = debug('hopr-chatbot:firebase:error')

type FirebaseDatabaseOptions = {
    databaseUrl: string
}

export default class FirebaseDatabase {
    databaseUrl: string

    constructor(options: FirebaseDatabaseOptions) {
        this.databaseUrl = options.databaseUrl
    }

    public async getSchema(schema: string) {
        try {
          log(`- getSchema | Retrieving schema ${schema} from ${this.databaseUrl}`)
          const response = await fetch(`${this.databaseUrl}${schema}.json`)
            .catch(err => error(`- getSchema | fetch :: Error retrieve data from database`, err))
          const json = await response.json()
            .catch(err => error(`- getSchema | json :: Error parsing data from response`, err))
          log(`- getSchema | Retrieved json ${JSON.stringify(json)}`)
          return json
        } catch (err) {
          error(`- getSchema | catch :: Error retrieving data`, err)
          return {}
        }
    }

    public async getTable(schema: string, table: string) {
        log(`- getTable | Retrieving table ${table} within schema ${schema} from ${this.databaseUrl}`)
        const response = await fetch(`${this.databaseUrl}${schema}/${table}.json`)
        const json = await response.json()
        log(`- getSchema | Retrieved json ${json}`)
        return json
    }
}