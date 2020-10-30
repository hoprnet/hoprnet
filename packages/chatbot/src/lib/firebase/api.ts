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

    public async getTable(schema: string, table: string) {
        log(`- getTable | Retrieving table ${table} within schema ${schema} from db ${this.databaseUrl}`)
        const response = await fetch(`${this.databaseUrl}/${schema}/${table}.json`)
        const json = await response.json()
        log(`- getTable | Retrieved json ${json}`)
        return json
    }
}