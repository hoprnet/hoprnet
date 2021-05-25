import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { join } from 'path'
import { readdir, writeFile } from 'fs/promises'

const DEPLOYMENTS_PATH = join(__dirname, '..', 'deployments')

// deployments are done by `hardhat-deploy`
// after a deployment, `hardhat-deploy` populates `deployments`
// folder with various artifacts, this task loops through `deployments`
// folder and removes data that are optional & will end up being commited
const main: DeployFunction = async function (_hre: HardhatRuntimeEnvironment) {
  const networks = await readdir(DEPLOYMENTS_PATH)

  for (const network of networks) {
    const contracts = await readdir(join(DEPLOYMENTS_PATH, network)).then((files) =>
      files.filter((file) => file.endsWith('.json'))
    )

    for (const contract of contracts) {
      const filePath = join(DEPLOYMENTS_PATH, network, contract)
      const data = require(filePath)
      const slimmed = {
        address: data.address,
        transactionHash: data.transactionHash,
        abi: data.abi
      }

      await writeFile(filePath, JSON.stringify(slimmed, null, 2))
    }
  }
}

main.runAtTheEnd = true

export default main
