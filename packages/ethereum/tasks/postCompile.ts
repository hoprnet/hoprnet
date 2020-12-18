import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { Export } from 'hardhat-deploy/types'
import { join } from 'path'
import { promises, existsSync } from 'fs'

const { readFile, writeFile, mkdir } = promises
const CHAIN_DIR = join(__dirname, '..', 'chain')
const ABIS_DIR = join(CHAIN_DIR, 'abis')

/**
 * Updates chain/abis folder after compilation.
 */
async function main(_params, { run, config }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const fileDir = join(config.paths.cache, 'deployed_contracts.json')

  // use hardhat-deploy export to get data about the contracts
  await run('export', { export: fileDir })

  // read exported data of this network
  const result: Export = JSON.parse(await readFile(fileDir, { encoding: 'utf-8' }))

  if (!existsSync(ABIS_DIR)) {
    await mkdir(ABIS_DIR, { recursive: true })
  }

  // store abi files
  return Promise.all(
    Object.entries(result.contracts).map(([contractName, { abi }]) => {
      return writeFile(join(ABIS_DIR, `${contractName}.json`), JSON.stringify(abi, null, 2))
    })
  )
}

export default main
