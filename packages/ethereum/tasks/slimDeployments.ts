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
    process.env['DEPLOY_LABEL'] ?? 'default',
    hre.network.name === 'hardhat' ? 'localhost' : hre.network.name
  )
  const contracts = (await readdir(basePath)).filter((file) => file.endsWith('.json'))

  for (const contract of contracts) {
    const filePath = join(basePath, contract)
    const data = require(filePath)
    const contractName = contract.replace('.json', '')
    const compilerData = await hre.artifacts.getBuildInfo(`contracts/${contractName}.sol:${contractName}`)
    const slimmed = {
      address: data.address,
      transactionHash: data.transactionHash,
      blockNumber: data.receipt.blockNumber,
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

export default main
