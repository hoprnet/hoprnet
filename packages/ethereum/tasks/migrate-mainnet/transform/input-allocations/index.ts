import type { AllocationsRaw } from '../types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import { transformAllocations } from '../utils'

const readdir = promisify(_readdir)
const readFile = promisify(_readFile)
const writeFile = promisify(_writeFile)
const INPUT_DIR = __dirname
const OUTPUT_DIR = join(__dirname, '..', 'output')

export default async () => {
  const files = await readdir(INPUT_DIR).then((res) => res.filter((name) => name.includes('.csv')))
  const contents = await Promise.all(
    files.map(async (name) => {
      return {
        name: name.replace('.csv', ''),
        content: await readFile(join(INPUT_DIR, name), { encoding: 'utf-8' })
      }
    })
  )

  const rawAllocations: AllocationsRaw = {
    accounts: [],
    amounts: []
  }

  for (const content of contents) {
    const allocation = csvToAllocations(content.content)
    rawAllocations.accounts = rawAllocations.accounts.concat(allocation.accounts)
    rawAllocations.amounts = rawAllocations.amounts.concat(allocation.amounts)
  }

  const allocations = transformAllocations('bounties', rawAllocations)

  await writeFile(join(OUTPUT_DIR, `${allocations.name}-allocations.json`), JSON.stringify(allocations, null, 2))
}

const csvToAllocations = (csv: string): AllocationsRaw => {
  const result: AllocationsRaw = {
    accounts: [],
    amounts: []
  }
  const rows = csv.split('\n')

  for (const row of rows) {
    const [address, , total] = row.split(',')
    result.accounts.push(address)
    result.amounts.push(total)
  }

  return result
}
