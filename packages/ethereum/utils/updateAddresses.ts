import { join } from 'path'
import { promises } from 'fs'

const { stat, mkdir, readFile, writeFile } = promises
const DIR = join(__dirname, '..', 'chain', 'addresses.json')

async function main(
  network: string,
  addresses: {
    [name: string]: string
  }
): Promise<void> {
  if (!(await stat(DIR)).isDirectory()) {
    await mkdir(DIR, { recursive: true })
  }

  let json: any

  try {
    json = JSON.parse(await readFile(DIR, { encoding: 'utf-8' }))
  } catch {
    json = {}
  }

  json[network] = addresses

  return writeFile(DIR, JSON.stringify(json, null, 2))
}

export default main
