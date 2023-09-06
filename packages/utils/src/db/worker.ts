import { default as Sqlite, type Database, Statement } from 'better-sqlite3'
import workerpool from 'workerpool'

import { debug } from '../process/index.js'
import { u8aToHex, stringToU8a } from '../u8a/index.js'

const log = debug(`hopr-core:db:worker`)

let backend: Database
let iterStatement: Statement
let getStatement: Statement
let putStatement: Statement
let removeStatement: Statement

function getIterStatement(): Statement {
    if (!iterStatement) {
        iterStatement = backend.prepare('SELECT value FROM kv2 WHERE key LIKE ?')
    }
    return iterStatement
}

function getPutStatement(): Statement {
      if (!putStatement) {
          putStatement = backend.prepare(
              'INSERT INTO kv2 (key, value) VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value=excluded.value'
          )
      }
      return putStatement
  }

  function getGetStatement():  Statement {
      if (!getStatement) {
    getStatement = backend.prepare('SELECT value FROM kv2 WHERE key = ?')
      }
      return getStatement
  }

  function getRemoveStatement(): Statement {
      if (!removeStatement) {
    removeStatement = backend.prepare('DELETE FROM kv2 WHERE key = ?')
      }
      return removeStatement
  }

let dbPath: string
let enableDebugLog: boolean = false

console.log(process.argv)
if (process.argv.length > 2) {
    dbPath = process.argv[2]
} else {
    log('ERROR: parameter dbPath missing')
    process.exit(1)
}
if (process.argv.length > 3) {
    enableDebugLog = process.argv[3] === 'true'
}

// open database connection
const options = {}
if (enableDebugLog) {
      options['verbose'] = log
}
const dbPathFull = dbPath + '/db.sqlite'
log(`opening database connection to ${dbPathFull} with options ${JSON.stringify(options)}`)
backend = new Sqlite(dbPathFull, options)

// setup connection parameters
backend.pragma('journal_mode = WAL')
backend.pragma('synchronous = normal')
backend.pragma('auto_vacuum = full')
backend.pragma('page_size = 4096')
backend.pragma('cache_size = -4000')

function put(key: Uint8Array, value: Uint8Array): void {
    const k = u8aToHex(key)
    backend.transaction(() => getPutStatement().run(k, u8aToHex(value)))()
}

function get(key: Uint8Array): Uint8Array | undefined {
    const k = u8aToHex(key)
    const tx = backend.transaction(() => getGetStatement().get(k))
    const row = tx()
    if (row) {
        const value = row['value']
        return stringToU8a(value)
    }
    return undefined
}

function remove(key: Uint8Array): void {
    const k = u8aToHex(key)
    backend.transaction(() => getRemoveStatement().run(k))()
}

function batch(ops: Array<any>): void {
    backend.transaction(() => {
        ops.forEach((op) => {
            if (op.type === 'put') {
                getPutStatement().run(u8aToHex(op.key), u8aToHex(op.value))
            } else if (op.type === 'del') {
                getRemoveStatement().run(u8aToHex(op.key))
            } else {
                throw new Error(`Unsupported operation type: ${JSON.stringify(op)}`)
            }
        })
    })()
}

function iterValues(prefix: Uint8Array, _suffix: number): Uint8Array[] {
    const k = u8aToHex(prefix)
    const tx = backend.transaction(() => getIterStatement().all(`${k}%`))
    const rows = tx()
    return rows.map((r) => stringToU8a(r['value']))
}

function exec(stmt: string): void {
    backend.exec(stmt)
}

function close(): void {
    log('Closing database')
    backend.close()
}

log(`joining worker pool`)
workerpool.worker({
    get,
    put,
    remove,
    batch,
    iterValues,
    exec,
    close
});
