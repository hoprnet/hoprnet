import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import { existsSync, mkdirSync } from 'fs'
import path from 'path'
import { VERSION } from './constants'
import type { HoprOptions } from '.'
import Debug from 'debug'

const log = Debug(`hopr-core:db`)

const defaultDBPath = (): string => {
  return path.join(process.cwd(), 'db', VERSION, 'node')
}

export function openDatabase(options: HoprOptions): LevelUp {
  let dbPath: string
  if (options.dbPath) {
    dbPath = options.dbPath
  } else {
    dbPath = defaultDBPath()
  }

  dbPath = path.resolve(dbPath)

  log('using db at ', dbPath)
  if (!existsSync(dbPath)) {
    log('db does not exist, creating?:', options.createDbIfNotExist)
    if (options.createDbIfNotExist) {
      mkdirSync(dbPath, { recursive: true })
    } else {
      throw new Error('Database does not exist: ' + dbPath)
    }
  }

  return levelup(leveldown(dbPath))
}
