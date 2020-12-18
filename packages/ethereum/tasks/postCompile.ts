import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { MultiExport } from 'hardhat-deploy/types'
import { join } from 'path'
import { promises, existsSync } from 'fs'

const { mkdir, writeFile } = promises
const OUTPUT_DIR = join(__dirname, '..', 'chain', 'abis')

/**
 * Updates 'chain/abis/X.json' with the latest ABI
 */
async function main(_params, { run, config }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const inputDir = join(config.paths.cache, 'export_all.json')

  // use hardhat-deploy export to get data about the contracts
  await run('export', { exportAll: inputDir })
  // read exported data of this network
  const exportResult: MultiExport = require(inputDir)

  // ensure chain folder is created
  if (!existsSync(OUTPUT_DIR)) {
    await mkdir(OUTPUT_DIR, { recursive: true })
  }

  const abis: {
    [contractName: string]: any[]
  } = {}

  // loop through exported result and retrieve what we need
  for (const perChainId of Object.values(exportResult)) {
    for (const { name: networkName, contracts } of Object.values(perChainId)) {
      // no need to do this for hardhat network
      if (networkName === 'hardhat') continue

      for (const [contractName, { abi }] of Object.entries(contracts)) {
        abis[contractName] = abi
      }
    }
  }

  // store updated data.json
  return Promise.all(
    Object.entries(abis).map(([contractName, abi]) => {
      return writeFile(join(OUTPUT_DIR, `${contractName}.json`), JSON.stringify(abi, null, 2))
    })
  )
}

export default main
