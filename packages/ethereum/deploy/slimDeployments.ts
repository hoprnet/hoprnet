import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { join } from 'path'
import { readdir, writeFile } from 'fs/promises'

// deployments are done by `hardhat-deploy`
// after a deployment, `hardhat-deploy` populates `deployments`
// folder with various artifacts, this task loops through `deployments`
// folder and removes data that are optional & will end up being commited
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const basePath = join(
    __dirname,
    '..',
    'deployments',
    hre.environment,
    hre.network.name
  )

  let contracts: string[]

  try {
    contracts = (await readdir(basePath)).filter((filename: string) => filename.endsWith('.json'))
  } catch (err) {
    // Ignore missing deployments in unit tests
    if (hre.network.name === 'hardhat' && err.code === 'ENOENT') {
      return
    }

    throw err
  }

  for (const contract of contracts) {
    const filePath = join(basePath, contract)
    const data = require(filePath)
    const contractName = contract.replace('.json', '')
    const compilerData =
      (await hre.artifacts.getBuildInfo(`contracts/${contractName}.sol:${contractName}`)) ?? data.compilerData
    const slimmed = {
      address: data.address,
      transactionHash: data.transactionHash,
      blockNumber: data.receipt ? data.receipt.blockNumber : data.blockNumber,
      metadata: {
        solcVersion: compilerData.solcVersion,
        input: compilerData.input
      },
      abi: data.abi
    }

    await writeFile(filePath, JSON.stringify(slimmed, null, 2))
  }
}

main.runAtTheEnd = true
main.dependencies = ['preDeploy']
main.tags = ['slimDeployments']

export default main
