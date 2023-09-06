import os from 'os'

import { stat, mkdir, rm } from 'fs/promises'
import {default as workerpool, type WorkerPool}  from 'workerpool'
import path from 'path';
import { fileURLToPath } from 'url';
import { debug } from '../process/index.js'
import { u8aToHex, stringToU8a } from '../u8a/index.js'

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const log = debug(`hopr-core:db:master`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

const NETWORK_KEY = encoder.encode('network_id')

export class QueryResults {
  private results: Uint8Array[] = []

  public push(item: Uint8Array) {
    this.results.push(item)
  }

  public length(): number {
    return this.results.length
  }

  public get(pos: number): Uint8Array{
    return this.results[pos]
  }
}

export class SqliteDb {
  private workerPool: WorkerPool
  private dbPath: string

  constructor() {
  }

  public async init(initializeDb: boolean, dbPath: string, forceCreate: boolean = false, networkId: string = '') {
    this.dbPath = dbPath
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
        log('db directory does not exist, creating?:', initializeDb)
        if (initializeDb) {
          await mkdir(dbPath, { recursive: true })
          setNetwork = true
        } else {
          throw new Error(`Database does not exist: ${dbPath}`)
        }
      }
    }

    log('starting database worker threads')
    this.startWorkers()

    await this.prepare(setNetwork, networkId)
  }

  private startWorkers(): void {
    // start minimum 2 workers, maximum 1 less than CPU cores
    const workerCount = Math.max(2, os.cpus().length - 1)

    this.workerPool = workerpool.pool(__dirname + '/worker.js', {
      workerType: 'thread',
      minWorkers: 'max',
      maxWorkers: workerCount,
      workerThreadOpts: {argv: [this.dbPath, process.env.HOPRD_DB_DEBUG_LOG]},
    });
  }

  public async prepare(setNetwork: boolean = false, networkId: string = ''): Promise<void> {
    // ensure latest schema is used
    await this.workerPool.exec('exec', ['CREATE TABLE IF NOT EXISTS kv2 (key TEXT PRIMARY KEY, value BLOB)'])
    await this.workerPool.exec('exec', ['DROP TABLE IF EXISTS kv'])

    if (setNetwork) {
      log(`setting network id ${networkId} to db`)
      await this.put(NETWORK_KEY, encoder.encode(networkId))
    } else {
      let storedNetworkId = await this.get(NETWORK_KEY)
      let decodedStoredNetworkId = storedNetworkId !== undefined ? undefined : decoder.decode(storedNetworkId)

      const hasNetworkKey = decodedStoredNetworkId !== undefined && decodedStoredNetworkId === networkId

      if (!hasNetworkKey) {
        throw new Error(`invalid db network id: ${decodedStoredNetworkId} (expected: ${networkId})`)
      }
    }
  }

  public async put(key: Uint8Array, value: Uint8Array): Promise<void> {
    await this.workerPool.exec('put', [u8aToHex(key), u8aToHex(value)])
  }

  public async get(key: Uint8Array): Promise<Uint8Array | undefined> {
    const result = await this.workerPool.exec('get', [u8aToHex(key)])
    log("DB GET: ", u8aToHex(key), " - ", result)
    if (result) {
      log("DB GET: ", u8aToHex(key), " - ", stringToU8a(result))
      return stringToU8a(result)
    }
    return undefined
  }

  public async remove(key: Uint8Array): Promise<void> {
    await this.workerPool.exec('remove', [u8aToHex(key)])
  }

  public async batch(ops: Array<any>): Promise<void> {
    const opsConverted = ops.map((o) => {
      const newOp = {}
      if (o['key']) {
        newOp['key'] = u8aToHex(o['key'])
      }
      if (o['value']) {
        newOp['value'] = u8aToHex(o['value'])
      }
      if (o['type']) {
        newOp['type'] = o['type'].toString()
      }
      return newOp
    })
    await this.workerPool.exec('batch', [opsConverted])
  }

  public async iterValues(prefix: Uint8Array, _suffix: number): Promise<QueryResults> {
    const results = await this.workerPool.exec('iterValues', [u8aToHex(prefix)])
    let qResults = new QueryResults()
    for (const r of results) {
      qResults.push(stringToU8a(r))
    }
    return qResults
  }

  public async close(): Promise<void> {
    log('Closing database')
    await this.workerPool.exec('close', [])
  }

  public async setNetworkId(network_id: string): Promise<void> {
    // conversion to Buffer done by `.put()` method
    await this.put(NETWORK_KEY, encoder.encode(network_id))
  }

  public async getNetworkId(): Promise<string | undefined> {
    // conversion to Buffer done by `.get()` method
    return decoder.decode(await this.get(NETWORK_KEY))
  }

  public async verifyNetworkId(expectedId: string): Promise<boolean> {
    const storedId = await this.getNetworkId()

    if (storedId == undefined) {
      return false
    }

    return storedId === expectedId
  }
}
