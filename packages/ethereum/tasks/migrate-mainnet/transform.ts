import type { HoprDistributorParams, Schedule, Allocations } from './types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'
import { toMultiplier } from '../../utils'

const readdir = promisify(_readdir)
const readFile = promisify(_readFile)
const writeFile = promisify(_writeFile)
const RAW_DIR = join(__dirname, 'raw')
const DATA_DIR = join(__dirname, 'data')
const MULTIPLIER = String(10 ** 6)
const { toWei, toBN, toChecksumAddress } = Web3.utils

type HoprDistributorParamsRaw = Omit<HoprDistributorParams, 'network'>
type ScheduleRaw = Omit<Schedule, 'name'>
type AllocationsRaw = Omit<Allocations, 'name'>

const transformHoprDistributorParams = (network: string, input: HoprDistributorParamsRaw): HoprDistributorParams => {
  return {
    network,
    startTime: String(new Date(input.startTime).getTime()),
    maxMintAmount: toWei(input.maxMintAmount, 'ether'),
    multisig: toChecksumAddress(input.multisig)
  }
}

const transformSchedule = (name: string, input: ScheduleRaw, multiplier: string): Schedule => {
  if (input.durations.length !== input.percents.length) {
    throw Error('Durations and percents length is not the same')
  }

  return {
    name,
    durations: input.durations.map((days) => String(durations.days(Number(days)))),
    percents: input.percents.map((percent) => {
      const result = toMultiplier(percent, multiplier)
      if (toBN(result).gt(toBN(multiplier))) {
        throw Error('Multiplied result is higher than multiplier')
      }

      return result
    })
  }
}

const transformAllocations = (name: string, input: AllocationsRaw): Allocations => {
  if (input.accounts.length !== input.amounts.length) {
    throw Error('Account and amounts length is not the same')
  }

  // add duplicates
  const balances = new Map<string, string>()
  for (let i = 0; i < input.accounts.length; i++) {
    const account = toChecksumAddress(input.accounts[i])
    const amount = toWei(input.amounts[i], 'ether')
    const balance = toBN(balances.get(account) ?? 0).add(toBN(amount))
    balances.set(account, balance.toString())
  }

  return {
    name,
    accounts: Array.from(balances.keys()),
    amounts: Array.from(balances.values())
  }
}

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

  const fileNames = await readdir(RAW_DIR).then((fileNames) =>
    fileNames.filter((fileName) => fileName.endsWith('.json'))
  )

  for (const fileName of fileNames) {
    const [name, _type] = fileName.split('-')
    const type = _type.replace('.json', '')

    if (!name || !type) {
      throw Error(`Invalid raw filename found ${fileName}`)
    } else if (!['params', 'schedule', 'allocations'].includes(type)) {
      throw Error(`Type ${type} from filename ${fileName} not found`)
    }

    const fileData = await readFile(join(RAW_DIR, fileName), { encoding: 'utf-8' })
    const json: HoprDistributorParamsRaw | ScheduleRaw | AllocationsRaw = JSON.parse(fileData)

    let transformed: HoprDistributorParams | Schedule | Allocations
    if (type === 'params') {
      transformed = transformHoprDistributorParams(name, json as HoprDistributorParamsRaw)
    } else if (type === 'schedule') {
      transformed = transformSchedule(name, json as ScheduleRaw, MULTIPLIER)
    } else if (type === 'allocations') {
      transformed = transformAllocations(name, json as AllocationsRaw)
    }

    await writeFile(join(DATA_DIR, fileName), JSON.stringify(transformed, null, 2))
  }
}
