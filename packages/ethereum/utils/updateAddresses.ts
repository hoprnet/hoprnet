import { join } from 'path'
import { promises, existsSync } from 'fs'

const { mkdir, readFile, writeFile } = promises
const DIR = join(__dirname, '..', 'chain')
const FILE_DIR = join(DIR, 'addresses.json')

async function main(
  network: string,
  addresses: {
    [name: string]: string
  }
): Promise<void> {
  if (!existsSync(DIR)) {
    await mkdir(DIR, { recursive: true })
  }

  let json: any

  try {
    json = JSON.parse(await readFile(FILE_DIR, { encoding: 'utf-8' }))
  } catch {
    json = {}
  }

  json[network] = addresses

  return writeFile(FILE_DIR, JSON.stringify(json, null, 2))
}

export default main
