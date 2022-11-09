import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { join } from 'path'
import { readdir } from 'fs/promises'
import { get } from 'https'

// request blockscout/etherscan if the address is already verified, by requesting the ABI of the said contract address. Only
// verified contract returns ABI with status '1'
const checkIfContractIsVerified = async (
  contractName: string,
  contractAddress: string,
  hre: HardhatRuntimeEnvironment
) => {
  // get verification endpoint
  const verificationConfig = await hre.run('verify:get-etherscan-endpoint')
  let verificationApiKey
  if (typeof hre.config.etherscan.apiKey === 'string') {
    verificationApiKey = `&apikey=${hre.config.etherscan.apiKey}`
  } else if (
    Object.keys(hre.config.etherscan.apiKey).some((key) => key === hre.network.name) &&
    hre.config.etherscan.apiKey[hre.network.name] &&
    hre.config.etherscan.apiKey[hre.network.name].length > 0
  ) {
    // if the contract is deployed on mainnet or goerli, contracts should be verified on etherscan
    // use hardhat-etherscan to verify: https://hardhat.org/hardhat-runner/plugins/nomiclabs-hardhat-etherscan
    verificationApiKey = `&apikey=${hre.config.etherscan.apiKey[hre.network.name]}`
  } else {
    // if the contract is deployed on xdai or sokol, use blockscout to verify
    // https://docs.blockscout.com/for-users/verifying-a-smart-contract
    verificationApiKey = ''
  }
  console.log(`Verifying contract ${contractName} at ${contractAddress}`)

  const url = `${verificationConfig.urls.apiURL}?module=contract&action=getabi&address=${contractAddress}${verificationApiKey}`

  return new Promise((resolve, reject) => {
    const req = get(url, (res) => {
      let rawData = ''
      res.on('data', (chunk) => {
        rawData += chunk
      })
      res.on('end', () => {
        resolve(JSON.parse(rawData))
      })
    })
    req.on('error', (err) => {
      reject(err)
    })
    req.end()
  })
}

/**
 * It runs once deployment has finished
 */
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  // verify smart contract to etherscan
  console.log(`postDeploy with right tag ${hre.network.tags.etherscan}`)
  if (!hre.network.tags.etherscan) {
    console.log(`Should skip verify task`)
    return
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

    console.log('contractName', contractName)
    console.log('contractAddress', contractAddress)

    let result
    try {
      // call explorer API. TODO: add throttle
      result = await checkIfContractIsVerified(contractName, contractAddress, hre)
      console.log('check if contract has been verified', result)
    } catch (error) {
      console.error(`  >> Error when checking verified contract with API ${error}`)
      continue
    }

    if ((result as any).status === '0') {
      // When {"message": "Contract source code not verified", "status": "0"} continue with the verification
      try {
        await hre.run('verify:verify', {
          address: contractAddress,
          constructorArguments: data.args,
          listNetworks: true
        })
      } catch (error) {
        if (error.message.includes('Reason: Already Verified')) {
          console.log(`  Contract ${contractName} has been already verified!`)
        } else if (error.message.includes('arguments were provided instead')) {
          console.error(`  >> Contract ${contractName} misses argument(s)!`)
        } else {
          throw error
        }
      }
    } else if ((result as any).status === '1') {
      // When {"status": "1"} skip the verification
      console.log(`  Contract ${contractName} is already verified.`)
    } else {
      console.error(`  >> Unexpected status code ${(result as any).status} when verifying contract`)
    }
  }
}

main.runAtTheEnd = true
main.dependencies = ['preDeploy']
main.tags = ['postDeploy']
main.skip = async (env: HardhatRuntimeEnvironment) => !env.network.tags.etherscan

export default main
