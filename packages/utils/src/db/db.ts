import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process/index.js'

import fs from 'fs'
import { stringToU8a, u8aConcat, u8aToHex } from '../u8a/index.js'
import { AbstractLevel } from 'abstract-level'
import { MemoryLevel } from 'memory-level'
const SqliteLevel = (await import('sqlite-level')).default.SqliteLevel

const log = debug(`hopr-core:db`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

const NETWORK_KEY = encoder.encode('network_id')

export class LevelDb {
  public backend: AbstractLevel<string | Uint8Array | Buffer>

  constructor() {
    // unless initialized with a specific db path, memory version is used
    this.backend = new MemoryLevel()
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

    this.backend = new SqliteLevel({ filename: dbPath + '/db.sqlite' })

    // Fully initialize database
    await this.backend.open()

    if (setNetwork) {
      log(`setting network id ${networkId} to db`)
      await this.put(NETWORK_KEY, encoder.encode(networkId))
    } else {
      let storedNetworkId = await this.maybeGet(NETWORK_KEY)
      let decodedStoredNetworkId = storedNetworkId !== undefined ? undefined : decoder.decode(storedNetworkId)

      const hasNetworkKey = decodedStoredNetworkId !== undefined && decodedStoredNetworkId === networkId

      if (!hasNetworkKey) {
        throw new Error(`invalid db network id: ${decodedStoredNetworkId} (expected: ${networkId})`)
      }
    }
  }

  public async put(key: Uint8Array, value: Uint8Array): Promise<void> {
    let k = u8aToHex(key)
    await this.backend.del(k) // Delete first in case the value already exists
    return await this.backend.put(k, u8aToHex(value))
  }

  public async get(key: Uint8Array): Promise<Uint8Array> {
    return stringToU8a(await this.backend.get(u8aToHex(key)))
  }

  public async remove(key: Uint8Array): Promise<void> {
    await this.backend.del(u8aToHex(key))
  }

  public async batch(ops: Array<any>, wait_for_write = true): Promise<void> {
    const options: { sync: boolean } = {
      sync: wait_for_write
    }

    let batch = this.backend.batch()
    for (const op of ops) {
      if (!op.hasOwnProperty('type') || !op.hasOwnProperty('key')) {
        throw new Error('Invalid operation, missing key or type: ' + JSON.stringify(op))
      }

      if (op.type === 'put') {
        batch.del(u8aToHex(op.key)) // We must try to delete first then insert (in case of updates)
        batch.put(u8aToHex(op.key), u8aToHex(op.value))
      } else if (op.type === 'del') {
        batch.del(u8aToHex(op.key))
      } else {
        throw new Error(`Unsupported operation type: ${JSON.stringify(op)}`)
      }
    }

    await batch.write(options)
  }

  public async maybeGet(key: Uint8Array): Promise<Uint8Array | undefined> {
    try {
      return await this.get(key)
    } catch (err) {
      if (err.type === 'NotFoundError' || err.notFound) {
        return undefined
      }
      throw err
    }
  }

  public iterValues(prefix: Uint8Array, suffixLength: number): AsyncIterable<Uint8Array> {
    return this.iter(prefix, suffixLength)
  }

  protected async *iter(prefix: Uint8Array, suffixLength: number): AsyncIterable<Uint8Array> {
    const firstPrefixed = u8aConcat(prefix, new Uint8Array(suffixLength).fill(0x00))
    const lastPrefixed = u8aConcat(prefix, new Uint8Array(suffixLength).fill(0xff))

    for await (const [_key, chunk] of this.backend.iterator({
      // LevelDB does not support Uint8Arrays, always convert to Buffer
      gte: u8aToHex(firstPrefixed),
      lte: u8aToHex(lastPrefixed),
      keys: false
    }) as any) {
      yield stringToU8a(chunk)
    }
  }

  public async close() {
    log('Closing database')
    return await this.backend.close()
  }

  public async dump(destFile: string) {
    log(`Dumping current database to ${destFile}`)
    let dumpFile = fs.createWriteStream(destFile, { flags: 'a' })
    for await (const [key_hex, value_hex] of this.backend.iterator()) {
      let key = stringToU8a(key_hex)
      let out = ''
      while (key.length > 0) {
        const nextDelimiter = key.findIndex((v: number) => v == 0x2d) // 0x2d ~= '-'

        if (key.subarray(0, nextDelimiter).every((v: number) => v >= 32 && v <= 126)) {
          out += decoder.decode(key.subarray(0, nextDelimiter))
        } else {
          out += u8aToHex(key.subarray(0, nextDelimiter))
        }

        if (nextDelimiter < 0) {
          break
        } else {
          key = (key as Buffer).subarray(nextDelimiter + 1)
        }
      }
      dumpFile.write(out + ',' + value_hex + '\n')
    }
    dumpFile.close()
  }

  public async setNetworkId(network_id: string): Promise<void> {
    // conversion to Buffer done by `.put()` method
    await this.put(NETWORK_KEY, encoder.encode(network_id))
  }

  public async getNetworkId(): Promise<string> {
    // conversion to Buffer done by `.get()` method
    return decoder.decode(await this.maybeGet(NETWORK_KEY))
  }

  public async verifyNetworkId(expectedId: string): Promise<boolean> {
    const storedId = await this.getNetworkId()

    if (storedId == undefined) {
      return false
    }

    return storedId === expectedId
  }
}
