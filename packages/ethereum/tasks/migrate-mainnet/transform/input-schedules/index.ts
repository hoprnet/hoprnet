import type {
  HoprDistributorParams,
  Schedule,
  Allocations,
  HoprDistributorParamsRaw,
  ScheduleRaw,
  AllocationsRaw
} from '../types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import { transformHoprDistributorParams, transformSchedule } from '../utils'

const readdir = promisify(_readdir)
const readFile = promisify(_readFile)
const writeFile = promisify(_writeFile)
const INPUT_DIR = __dirname
const OUTPUT_DIR = join(__dirname, '..', 'output')

export default async (): Promise<void> => {
  const fileNames = await readdir(INPUT_DIR).then((fileNames) =>
    fileNames.filter((fileName) => fileName.endsWith('.json'))
  )

  for (const fileName of fileNames) {
    const [name, _type] = fileName.split('-')
    const type = _type.replace('.json', '')

    if (!name || !type) {
      throw Error(`Invalid raw filename found ${fileName}`)
    } else if (!['params', 'schedule', 'allocation'].includes(type)) {
      throw Error(`Type ${type} from filename ${fileName} not found`)
    }

    const fileData = await readFile(join(INPUT_DIR, fileName), { encoding: 'utf-8' })
    const json: HoprDistributorParamsRaw | ScheduleRaw | AllocationsRaw = JSON.parse(fileData)

    let transformed: HoprDistributorParams | Schedule | Allocations
    if (type === 'params') {
      transformed = transformHoprDistributorParams(name, json as HoprDistributorParamsRaw)
    } else if (type === 'schedule') {
      transformed = transformSchedule(name, json as ScheduleRaw)
    }

    await writeFile(join(OUTPUT_DIR, fileName), JSON.stringify(transformed, null, 2))
  }
}
