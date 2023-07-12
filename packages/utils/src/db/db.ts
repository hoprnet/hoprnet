import levelup, { type LevelUp } from 'levelup'
import leveldown from 'leveldown'
import MemDown from 'memdown'
import { stat, mkdir, rm } from 'fs/promises'
import { debug } from '../process/index.js'

import fs from 'fs'
import { u8aConcat, u8aToHex } from '../u8a/index.js'
// import type { IteratedHash } from '../../../core/lib/core_crypto.js'

const log = debug(`hopr-core:db`)

const encoder = new TextEncoder()
const decoder = new TextDecoder()

const NETWORK_KEY = encoder.encode('network_id')

export class LevelDb {
  public backend: LevelUp

  constructor() {
    // unless initialized with a specific db path, memory version is used
    this.backend = new levelup(MemDown())
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

    // CommonJS / ESM issue
    // @ts-ignore
    this.backend = levelup(leveldown(dbPath))

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
    // LevelDB does not support Uint8Arrays, always convert to Buffer
    return await this.backend.put(
      Buffer.from(key.buffer, key.byteOffset, key.byteLength),
      Buffer.from(value.buffer, value.byteOffset, value.byteLength)
    )
  }

  public async get(key: Uint8Array): Promise<Uint8Array> {
    // LevelDB does not support Uint8Arrays, always convert to Buffer
    const value = await this.backend.get(Buffer.from(key.buffer, key.byteOffset, key.byteLength))

    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
  }

  public async remove(key: Uint8Array): Promise<void> {
    await this.backend.del(Buffer.from(key.buffer, key.byteOffset, key.byteLength))
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
        // LevelDB does not support Uint8Arrays, always convert to Buffer
        batch.put(
          Buffer.from(op.key, op.key.byteOffset, op.key.byteLength),
          Buffer.from(op.value, op.value.byteOffset, op.value.byteLength)
        )
      } else if (op.type === 'del') {
        // LevelDB does not support Uint8Arrays, always convert to Buffer
        batch.del(Buffer.from(op.key, op.key.byteOffset, op.key.byteLength))
      } else {
        throw new Error(`Unsupported operation type: ${JSON.stringify(op)}`)
      }
    }

    await batch.write(options)
  }

  public async maybeGet(key: Uint8Array): Promise<Uint8Array | undefined> {
    try {
      // Conversion to Buffer done by `.get()` method
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
      gte: Buffer.from(firstPrefixed.buffer, firstPrefixed.byteOffset, firstPrefixed.byteLength),
      lte: Buffer.from(lastPrefixed.buffer, lastPrefixed.byteOffset, lastPrefixed.byteLength),
      keys: false
    }) as any) {
      const obj: Uint8Array = new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength)

      yield obj
    }
  }

  public async close() {
    log('Closing database')
    return await this.backend.close()
  }

  public dump(destFile: string) {
    log(`Dumping current database to ${destFile}`)
    let dumpFile = fs.createWriteStream(destFile, { flags: 'a' })
    this.backend
      .createReadStream({ keys: true, keyAsBuffer: true, values: true, valueAsBuffer: true })
      .on('data', ({ key }) => {
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

          dumpFile.write(out + '\n')
        }
      })
      .on('end', function () {
        dumpFile.close()
      })
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
