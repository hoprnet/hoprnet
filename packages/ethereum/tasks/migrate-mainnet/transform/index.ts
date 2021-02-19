import type { HoprDistributorParams, Schedule, Allocations } from './types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import transformSchedules from './input-allocations'
import transformAllocations from './input-schedules'

const readFile = promisify(_readFile)
const DATA_DIR = join(__dirname, 'data')

export const getHoprDistributorParams = async (network: string): Promise<HoprDistributorParams> => {
  return JSON.parse(await readFile(join(DATA_DIR, `${network}-params.json`), { encoding: 'utf-8' }))
}

export const getSchedule = async (name: string): Promise<Schedule> => {
  return JSON.parse(await readFile(join(DATA_DIR, `${name}-schedule.json`), { encoding: 'utf-8' }))
}

export const getAllocations = async (name: string): Promise<Allocations> => {
  return JSON.parse(await readFile(join(DATA_DIR, `${name}-allocations.json`), { encoding: 'utf-8' }))
}

export default async (): Promise<void> => {
  console.log(`Transforming raw data into usable data`)

  await transformSchedules()
  await transformAllocations()
}
