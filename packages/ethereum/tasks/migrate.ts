import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'
import { singletons } from '@openzeppelin/test-helpers'
import { migrationOptions as allMigrationOptions, MigrationOptions } from '../utils/networks'
import updateAddresses from '../utils/updateAddresses'

const SECS_CLOSURE = Math.floor(durations.minutes(1) / 1e3)
const MAX_MINT_AMOUNT = Web3.utils.toWei('100000000', 'ether')
const MAX_MINT_DURATION = Math.floor(durations.days(365) / 1e3)
const SINGLE_FAUCET_MINTER = true
const EXTERNAL_FAUCET_MINTER = '0x1A387b5103f28bc6601d085A3dDC878dEE631A56'

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
    deployer,
    network: network.name,
    ...migrationOptions
  })

  // store addresses
  const addresses = {}

  // deploy ERC1820Registry
  // @TODO: when ERC1820Registry is missing in a public network, it cant be deployed
  console.log('Deploying ERC1820Registry')
  const ERC1820Registry = await singletons.ERC1820Registry(deployer)
  console.log(`Deployed or Found ERC1820Registry: ${ERC1820Registry.address}`)

  // deploy HoprToken
  const HoprToken = artifacts.require('HoprToken')
  console.log('Deploying hoprToken')
  const hoprToken = await HoprToken.new()
  console.log(`Deployed hoprToken: ${hoprToken.address}`)
  addresses['HoprToken'] = hoprToken.address
  const minterRole = await hoprToken.MINTER_ROLE()

  if (!migrationOptions.revokeRoles) {
    await hoprToken.grantRole(minterRole, deployer)
  }

  // deploy HoprChannels
  const HoprChannels = artifacts.require('HoprChannels')
  console.log('Deploying hoprChannels')
  const hoprChannels = await HoprChannels.new(hoprToken.address, SECS_CLOSURE)
  console.log(`Deployed hoprChannels: ${hoprChannels.address}`)
  addresses['HoprChannels'] = hoprChannels.address

  // deploy HoprMinter
  if (migrationOptions.mintUsing === 'minter') {
    const HoprMinter = artifacts.require('HoprMinter')
    console.log('Deploying hoprMinter')
    const hoprMinter = await HoprMinter.new(hoprToken.address, MAX_MINT_AMOUNT, MAX_MINT_DURATION)
    console.log(`Deployed hoprMinter: ${hoprMinter.address}`)
    addresses['HoprMinter'] = hoprMinter.address

    if (migrationOptions.revokeRoles) {
      await hoprToken.grantRole(minterRole, hoprMinter.address)
      await hoprToken.renounceRole(await hoprToken.MINTER_ROLE(), deployer)
      await hoprToken.renounceRole(await hoprToken.DEFAULT_ADMIN_ROLE(), deployer)
    }
  }

  // deploy HoprFaucet
  if (migrationOptions.mintUsing === 'faucet') {
    const HoprFaucet = artifacts.require('HoprFaucet')
    console.log('Deploying hoprFaucet')
    const hoprFaucet = await HoprFaucet.new(hoprToken.address, SINGLE_FAUCET_MINTER)
    console.log(`Deployed hoprFaucet: ${hoprFaucet.address}`)
    addresses['HoprFaucet'] = hoprFaucet.address
    const pauserRole = await hoprFaucet.PAUSER_ROLE()
    const minterRole = await hoprFaucet.MINTER_ROLE()

    if (!migrationOptions.revokeRoles) {
      await hoprFaucet.grantRole(pauserRole, deployer)
      await hoprFaucet.grantRole(minterRole, hoprFaucet.address)
    }
    // give 'owner' OR 'XDAI_FAUCET_ADDRESS' MINTER_ROLE and PAUSER_ROLE
    else {
      if (SINGLE_FAUCET_MINTER) {
        await hoprFaucet.grantRole(pauserRole, deployer)
        await hoprFaucet.grantRole(minterRole, EXTERNAL_FAUCET_MINTER)
      } else {
        await hoprFaucet.grantRole(pauserRole, deployer)
        await hoprFaucet.grantRole(minterRole, hoprFaucet.address)
      }
      await hoprToken.grantRole(minterRole, hoprFaucet.address)
    }
  }

  console.log('Updating addresses.json')
  await updateAddresses(network.name, addresses)
  console.log(`Updated addresses.json for ${network.name} network`)
}

export default main
