import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { join, basename } from 'path'
import { promises } from 'fs'

const { stat, mkdir, writeFile } = promises
const DIR = join(__dirname, '..', 'chain', 'abis')

async function main(_args: any, { artifacts }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  console.log('Extracting ABIs')

  if (!(await stat(DIR)).isDirectory()) {
    await mkdir(DIR, { recursive: true })
  }

  const paths = await artifacts.getArtifactPaths().then((paths) => {
    return paths.filter((path) => path.includes('hardhat/artifacts/contracts/'))
  })

  await Promise.all(
    paths.map(async (path) => {
      const name = basename(path)
      const artifact = require(path) || {}
      const abi = artifact.abi || []

      return writeFile(join(DIR, name), JSON.stringify(abi, null, 2))
    })
  )

  console.log('Extracted ABIs')
}

export default main
