import type { Allocations } from '../types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import csvtojson from 'csvtojson'
import { transformAllocations } from '../utils'

const readdir = promisify(_readdir)
const writeFile = promisify(_writeFile)
const INPUT_DIR = __dirname
const OUTPUT_DIR = join(__dirname, '..', 'output')

export default async () => {
  const files = await readdir(INPUT_DIR).then((res) => res.filter((name) => name.includes('.csv')))

  const allocations: {
    [name: string]: Allocations
  } = {}

  for (const fileName of files) {
    const name = fileName.replace('.csv', '')
    const [allocationName] = name.split('-')

    const json: {
      account: string
      amount: string
      total: string
    }[] = await csvtojson({ headers: ['account', 'amount', 'total'] }).fromFile(join(INPUT_DIR, fileName))

    if (!allocations[allocationName])
      allocations[allocationName] = {
        name: allocationName,
        accounts: [],
        amounts: []
      }

    allocations[allocationName].accounts = allocations[allocationName].accounts.concat(json.map((o) => o.account))
    allocations[allocationName].amounts = allocations[allocationName].amounts.concat(json.map((o) => o.amount))
  }

  for (const [name, data] of Object.entries(allocations)) {
    const formatted = transformAllocations(name, data)
    await writeFile(join(OUTPUT_DIR, `${name}-allocations.json`), JSON.stringify(formatted, null, 2))
  }
}
