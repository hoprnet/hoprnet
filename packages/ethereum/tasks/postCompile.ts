import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { join } from 'path'
import { promises, existsSync } from 'fs'

const { mkdir, readdir, writeFile } = promises
const OUTPUT_DIR = join(__dirname, '..', 'chain', 'abis')

/**
 * Updates 'chain/abis/X.json' with the latest ABI
 */
async function main(_params, { run, config }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const inputDir = join(config.paths.cache, 'artifacts')

  // use hardhat-deploy export to get data about the contracts
  await run('export-artifacts', { dest: inputDir })

  const artifacts = await readdir(inputDir).then((files) =>
    files.filter((file) => {
      return file.startsWith('Hopr')
    })
  )

  // ensure chain folder is created
  if (!existsSync(OUTPUT_DIR)) {
    await mkdir(OUTPUT_DIR, { recursive: true })
  }

  const abis: {
    [contractName: string]: any[]
  } = {}

  // loop through exported result and retrieve what we need
  for (const artifact of artifacts) {
    const { contractName, abi } = require(join(inputDir, artifact))
    abis[contractName] = abi
  }

  // store abi file
  return Promise.all(
    Object.entries(abis).map(([contractName, abi]) => {
      return writeFile(join(OUTPUT_DIR, `${contractName}.json`), JSON.stringify(abi, null, 2))
    })
  )
}

export default main
