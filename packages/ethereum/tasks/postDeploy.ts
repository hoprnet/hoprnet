import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction, Export } from 'hardhat-deploy/types'
import { join } from 'path'
import { promises, existsSync } from 'fs'

const { readFile, writeFile, mkdir } = promises
const CHAIN_DIR = join(__dirname, '..', 'chain')
const ADDRESSES_DIR = CHAIN_DIR
const ADDRESSES_FILE = join(CHAIN_DIR, 'addresses.json')

/**
 * Updates chain/addresses.json file after deployment.
 * 1. export contract data into 'hardhat/cache/deployed_contracts.json'
 * 2. export addresses to 'chain/addresses.json'
 * 3. verify smart contracts if possible
 * @param _params
 * @param hre
 */
async function main(_params, hre: HardhatRuntimeEnvironment) {
  const { run, network } = hre
  const fileDir = join(hre.config.paths.cache, 'deployed_contracts.json')

  // use hardhat-deploy export to get data about the contracts
  await run('export', { export: fileDir })
  // read exported data of this network
  const data: Export = JSON.parse(await readFile(fileDir, { encoding: 'utf-8' }))

  if (!existsSync(ADDRESSES_DIR)) {
    await mkdir(ADDRESSES_DIR, { recursive: true })
  }

  let prevAddresses: {
    [network: string]: {
      [contract: string]: string
    }
  }
  try {
    prevAddresses = JSON.parse(await readFile(ADDRESSES_FILE, { encoding: 'utf-8' }))
  } catch {
    prevAddresses = {}
  }

  for (const [contract, { address }] of Object.entries(data.contracts)) {
    if (!prevAddresses[network.name]) prevAddresses[network.name] = {}
    prevAddresses[network.name][contract] = address
  }

  await writeFile(ADDRESSES_FILE, JSON.stringify(prevAddresses, null, 2))

  // try to verify
  if (network.tags.etherscan) {
    await run('etherscan-verify')
  }
}

export default main
