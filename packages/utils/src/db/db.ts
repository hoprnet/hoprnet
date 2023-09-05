import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process/index.js'

import { u8aToHex } from '../u8a/index.js'
import { default as Sqlite, type Database, type Statement } from 'better-sqlite3'

const log = debug(`hopr-core:db`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

const NETWORK_KEY = encoder.encode('network_id')

export class Db {
  public backend: Database

  private removeStatement: Statement
  private putStatement: Statement
  private getStatement: Statement

  constructor() {
    this.backend = new Sqlite(':memory:')
  }

  public async init(initialize: boolean, dbPath: string, forceCreate: boolean = false, networkId: string) {
    let setNetwork = false

    log(`using db at ${dbPath}`)
    if (forceCreate) {
      log('force create - wipe old database and create a new')
      await rm(dbPath, { recursive: true, force: true })
      await mkdir(dbPath, { recursive: true })
      setNetwork = true
    } else {
      let exists = false

      try {
        exists = !(await stat(dbPath)).isDirectory()
      } catch (err: any) {
        if (err.code === 'ENOENT') {
          exists = false
        } else {
          // Unexpected error, therefore throw it
          throw err
        }
      }

      if (!exists) {
        log('db directory does not exist, creating?:', initialize)
        if (initialize) {
          await mkdir(dbPath, { recursive: true })
          setNetwork = true
        } else {
          throw new Error(`Database does not exist: ${dbPath}`)
        }
      }
    }

    // open database connection
    this.backend = new Sqlite(dbPath + '/db.sqlite', { verbose: console.log })

    this.open(setNetwork, networkId)
  }

  public open(setNetwork: boolean = false, networkId: string = ''): void {
    // setup connection parameters
    this.backend.pragma('journal_mode = WAL')
    this.backend.pragma('synchronous = normal')
    this.backend.pragma('auto_vacuum = full')
    this.backend.pragma('page_size = 4096')
    this.backend.pragma('cache_size = -4000')

    // setup prepared statements
    this.removeStatement = this.backend.prepare('DELETE FROM kv2 WHERE key = ?')
    this.putStatement = this.backend.prepare(
      'INSERT INTO kv2 (key, value) VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value=excluded.value'
    )
    this.getStatement = this.backend.prepare('SELECT value FROM kv2 WHERE key = ?')
    // ensure latest schema is used
    this.backend.exec('CREATE TABLE IF NOT EXISTS kv2 (key TEXT PRIMARY KEY, value BLOB)')
    this.backend.exec('DROP TABLE IF EXISTS kv')

    if (setNetwork) {
      log(`setting network id ${networkId} to db`)
      this.put(NETWORK_KEY, encoder.encode(networkId))
    } else {
      let storedNetworkId = this.get(NETWORK_KEY)
      let decodedStoredNetworkId = storedNetworkId !== undefined ? undefined : decoder.decode(storedNetworkId)

      const hasNetworkKey = decodedStoredNetworkId !== undefined && decodedStoredNetworkId === networkId

      if (!hasNetworkKey) {
        throw new Error(`invalid db network id: ${decodedStoredNetworkId} (expected: ${networkId})`)
      }
    }
  }

  public put(key: Uint8Array, value: Uint8Array): void {
    const k = u8aToHex(key)
    this.backend.transaction(() => this.putStatement.run(k, value.toString()))()
  }

  public get(key: Uint8Array): Uint8Array | undefined {
    const k = u8aToHex(key)
    const tx = this.backend.transaction(() => this.getStatement.get(k))
    const row = tx()
    if (row) {
      const value = row['value']
      return value
    }
    return undefined
  }

  public remove(key: Uint8Array): void {
    const k = u8aToHex(key)
    this.backend.transaction(() => this.removeStatement.run(k))()
  }

  public batch(ops: Array<any>): void {
    this.backend.transaction(() => {
      ops.forEach((op) => {
        if (op.type === 'put') {
          this.putStatement.run(op.key, op.value)
        } else if (op.type === 'del') {
          this.removeStatement.run(op.key)
        } else {
          throw new Error(`Unsupported operation type: ${JSON.stringify(op)}`)
        }
      })
    })()
  }

  public close(): void {
    log('Closing database')
    this.backend.close()
  }

  public setNetworkId(network_id: string): void {
    // conversion to Buffer done by `.put()` method
    this.put(NETWORK_KEY, encoder.encode(network_id))
  }

  public getNetworkId(): string | undefined {
    // conversion to Buffer done by `.get()` method
    return decoder.decode(this.get(NETWORK_KEY))
  }

  public verifyNetworkId(expectedId: string): boolean {
    const storedId = this.getNetworkId()

    if (storedId == undefined) {
      return false
    }

    return storedId === expectedId
  }
}
