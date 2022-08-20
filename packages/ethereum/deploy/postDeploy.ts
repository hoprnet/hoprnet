import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { join } from 'path'
import { readdir } from 'fs/promises'

/**
 * It runs once deployment has finished
 */
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  // verify smart contract to etherscan
  console.log(`postDeploy with right tag ${hre.network.tags.etherscan}`)
  if (!hre.network.tags.etherscan) {
    console.log(`Should skip verify task`)
    return;
  }

  const basePath = join(
    __dirname,
    '..',
    'deployments',
    hre.environment,
    hre.network.name === 'hardhat' ? 'localhost' : hre.network.name
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
    const contractAddress = data.address

    console.log(`Trying to verify ${contractName} with args ${data.args}`)
    await hre.run('verify:verify', {
      address: contractAddress,
      constructorArgs: data.args,
      listNetworks: true
    })

    // const compilerData =
    //   (await hre.artifacts.getBuildInfo(`contracts/${contractName}.sol:${contractName}`)) ?? data.compilerData
    // // sometimes not all contracts are deployed, depends on the deployment scripts
    // if (!compilerData) continue
    // const slimmed = {
    //   address: data.address,
    //   transactionHash: data.transactionHash,
    //   blockNumber: data.receipt ? data.receipt.blockNumber : data.blockNumber,
    //   metadata: {
    //     solcVersion: compilerData.solcVersion,
    //     input: compilerData.input
    //   },
    //   abi: data.abi
    // }

    // await writeFile(filePath, JSON.stringify(slimmed, null, 2))
  }
}

main.runAtTheEnd = true
main.dependencies = ['preDeploy']
main.tags = ['postDeploy']
main.skip = async (env: HardhatRuntimeEnvironment) => !env.network.tags.etherscan

export default main
