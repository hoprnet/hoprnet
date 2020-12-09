import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { Export } from 'hardhat-deploy/types'
import { join } from 'path'
import { promises, existsSync } from 'fs'

const { readFile, writeFile, mkdir } = promises
const CHAIN_DIR = join(__dirname, '..', 'chain')
const ABIS_DIR = join(CHAIN_DIR, 'abis')

async function main(_params, hre: HardhatRuntimeEnvironment) {
  const { run } = hre
  const fileDir = join(hre.config.paths.cache, 'deployed_contracts.json')

  // use hardhat-deploy export to get data about the contracts
  await run('export', { export: fileDir })

  // read exported data of this network
  const result: Export = JSON.parse(await readFile(fileDir, { encoding: 'utf-8' }))

  if (!existsSync(ABIS_DIR)) {
    await mkdir(ABIS_DIR, { recursive: true })
  }

  return Promise.all(
    Object.entries(result.contracts).map(([contractName, { abi }]) => {
      return writeFile(join(ABIS_DIR, `${contractName}.json`), JSON.stringify(abi, null, 2))
    })
  )
}

export default main
