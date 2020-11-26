import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { durations } from '@hoprnet/hopr-utils'
import { singletons } from '@openzeppelin/test-helpers'
import { migrationOptions as allMigrationOptions, MigrationOptions } from '../utils/networks'
import updateAddresses from '../utils/updateAddresses'

const SECS_CLOSURE = Math.floor(durations.minutes(1) / 1e3)

async function main(
  providedMigrationOptions: Partial<MigrationOptions>,
  { web3, network, artifacts }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const networkMigrationOptions = allMigrationOptions[network.name]
  if (!networkMigrationOptions) throw Error(`Could not found network config for network '${network.name}'.`)

  const migrationOptions: MigrationOptions = {
    shouldVerify: providedMigrationOptions.shouldVerify ?? networkMigrationOptions.shouldVerify,
    mintUsing: providedMigrationOptions.mintUsing ?? networkMigrationOptions.mintUsing,
    revokeRoles: providedMigrationOptions.revokeRoles ?? networkMigrationOptions.revokeRoles
  }

  const [deployer] = await web3.eth.getAccounts()
  console.log(deployer)

  console.log('Running task "migrate" with config:', {
    network: network.name,
    ...migrationOptions
  })

  // store addresses
  const addresses = {}

  // deploy ERC1820Registry
  console.log('Deploying ERC1820Registry')
  const ERC1820Registry = await singletons.ERC1820Registry(deployer)
  console.log(`Deployed or Found ERC1820Registry: ${ERC1820Registry.address}`)

  // deploy HoprToken
  const HoprToken = artifacts.require('HoprToken')
  console.log('Deploying hoprToken')
  const hoprToken = await HoprToken.new()
  console.log(`Deployed hoprToken: ${hoprToken.address}`)
  addresses['HoprToken'] = hoprToken.address

  // @TODO: add minter role to MS wallet
  // const minterRole = await hoprToken.MINTER_ROLE()
  // if (!migrationOptions.revokeRoles) {
  //   await hoprToken.grantRole(minterRole, deployer)
  // }

  // deploy HoprChannels
  const HoprChannels = artifacts.require('HoprChannels')
  console.log('Deploying hoprChannels')
  const hoprChannels = await HoprChannels.new(hoprToken.address, SECS_CLOSURE)
  console.log(`Deployed hoprChannels: ${hoprChannels.address}`)
  addresses['HoprChannels'] = hoprChannels.address

  console.log('Updating addresses.json')
  await updateAddresses(network.name, addresses)
  console.log(`Updated addresses.json for ${network.name} network`)
}

export default main
