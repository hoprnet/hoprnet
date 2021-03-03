import type { Allocations } from '../types'
import { promisify } from 'util'
import { readdir as _readdir, readFile as _readFile, writeFile as _writeFile } from 'fs'
import { join } from 'path'
import csvtojson from 'csvtojson'
import BN from 'bn.js'
import { transformAllocations } from '../utils'

const readdir = promisify(_readdir)
const writeFile = promisify(_writeFile)
const INPUT_DIR = __dirname
const OUTPUT_DIR = join(__dirname, '..', 'output')

export default async () => {
  const files = await readdir(INPUT_DIR).then((res) => res.filter((name) => name.includes('.csv')))

  const dir: {
    [name: string]: Allocations
  } = {}

  for (const fileName of files) {
    const name = fileName.replace('.csv', '')
    const [allocationName] = name.split('-')

    const json: {
      account: string
      amount: string
      total: string
    }[] = await csvtojson({ noheader: true, headers: ['account', 'amount', 'total'] }).fromFile(
      join(INPUT_DIR, fileName)
    )

    if (!dir[allocationName])
      dir[allocationName] = {
        name: allocationName,
        accounts: [],
        amounts: []
      }

    dir[allocationName].accounts = dir[allocationName].accounts.concat(json.map((o) => o.account))
    dir[allocationName].amounts = dir[allocationName].amounts.concat(json.map((o) => o.amount))
  }

  for (const [name, data] of Object.entries(dir)) {
    const formatted = transformAllocations(name, data)

    const sum = formatted.amounts.reduce((result, a) => result.add(new BN(String(a))), new BN(0))
    console.log(`SUM ${name}: %s`, sum.toString())

    const chunk = 450
    for (let i = 0; i < formatted.accounts.length; i += chunk) {
      const allocations: any = {
        name: `${name}-allocations-${i}.json`,
        accounts: formatted.accounts.slice(i, i + chunk),
        amounts: formatted.amounts.slice(i, i + chunk),
      }

      await writeFile(join(OUTPUT_DIR, allocations.name), JSON.stringify(allocations, null, 2))
    }
  }
}
