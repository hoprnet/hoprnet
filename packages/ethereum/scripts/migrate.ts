import Web3 from 'web3'
import { singletons } from '@openzeppelin/test-helpers'
import { durations } from '@hoprnet/hopr-utils'
import { migrationOptions as allMigrationOptions } from './utils/networks'

const SECS_CLOSURE = Math.floor(durations.minutes(1) / 1e3)
const MAX_MINT_AMOUNT = Web3.utils.toWei('100000000', 'ether')
const MAX_MINT_DURATION = Math.floor(durations.days(365) / 1e3)
const SINGLE_FAUCET_MINTER = true
const EXTERNAL_FAUCET_MINTER = '0x1A387b5103f28bc6601d085A3dDC878dEE631A56'

async function main(taskArguments, hre, runSuper) {
  const [deployer] = await web3.eth.getAccounts()

  const networkName = hre.network.name
  const migrationOptions = allMigrationOptions[networkName]
  if (!migrationOptions) throw Error(`Could not found network config for network '${hre.network.name}'.`)

  console.log('Migrating with config:', {
    networkName,
    migrationOptions
  })

  // deploy ERC1820Registry
  console.log('Deploying ERC1820Registry')
  await singletons.ERC1820Registry(deployer)
  console.log(`Deployed ERC1820Registry`)

  // deploy HoprToken
  const HoprToken = hre.artifacts.require('HoprToken')
  console.log('Deploying hoprToken')
  const hoprToken = await HoprToken.new({ from: deployer })
  console.log(`Deployed hoprToken: ${hoprToken.address}`)
  const minterRole = await hoprToken.MINTER_ROLE()

  if (hre.network.name === 'hardhat') {
    await hoprToken.grantRole(minterRole, deployer)
  }

  // deploy HoprChannels
  const HoprChannels = artifacts.require('HoprChannels')
  console.log('Deploying hoprChannels')
  const hoprChannels = await HoprChannels.new(hoprToken.address, SECS_CLOSURE)
  console.log(`Deployed hoprChannels: ${hoprChannels.address}`)

  // deploy HoprMinter
  if (migrationOptions.mintUsing === 'minter') {
    const HoprMinter = artifacts.require('HoprMinter')
    console.log('Deploying hoprMinter')
    const hoprMinter = await HoprMinter.new(hoprToken.address, MAX_MINT_AMOUNT, MAX_MINT_DURATION)
    console.log(`Deployed hoprMinter: ${hoprMinter.address}`)

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
    const pauserRole = await hoprFaucet.PAUSER_ROLE()
    const minterRole = await hoprFaucet.MINTER_ROLE()

    if (hre.network.name === 'hardhat') {
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
}

export default main
