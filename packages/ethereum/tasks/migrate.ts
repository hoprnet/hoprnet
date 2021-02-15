import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'
import { singletons } from '@openzeppelin/test-helpers'
import updateAddresses from '../utils/updateAddresses'

// default values for testnets
const SECS_CLOSURE = Math.floor(durations.minutes(1) / 1e3)
const SECS_DISTRIBUTOR = Math.floor(durations.days(1) / 1e3)
const MAX_MINT_AMOUNT = Web3.utils.toWei('100000000', 'ether')

async function main(
  _options: any,
  { web3, network, artifacts }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const [deployer] = await web3.eth.getAccounts()

  console.log('Running task "migrate" with config:', {
    deployer,
    network: network.name
  })

  // store addresses
  const addresses = {}

  // deploy ERC1820Registry
  // when ERC1820Registry is missing in a public network, it cant be deployed
  // fixed in `ethereum-refactor`
  console.log('Deploying ERC1820Registry')
  const ERC1820Registry = await singletons.ERC1820Registry(deployer)
  console.log(`Deployed or Found ERC1820Registry: ${ERC1820Registry.address}`)

  // deploy HoprToken
  const HoprToken = artifacts.require('HoprToken')
  console.log('Deploying hoprToken')
  const hoprToken = await HoprToken.new()
  console.log(`Deployed hoprToken: ${hoprToken.address}`)
  addresses['HoprToken'] = hoprToken.address

  // deploy HoprChannels
  const HoprChannels = artifacts.require('HoprChannels')
  console.log('Deploying hoprChannels')
  const hoprChannels = await HoprChannels.new(hoprToken.address, SECS_CLOSURE)
  console.log(`Deployed hoprChannels: ${hoprChannels.address}`)
  addresses['HoprChannels'] = hoprChannels.address

  // deploy HoprDistributor
  const HoprDistributor = artifacts.require('HoprDistributor')
  console.log('Deploying hoprDistributor')
  const hoprDistributor = await HoprDistributor.new(hoprToken.address, SECS_DISTRIBUTOR, MAX_MINT_AMOUNT)
  console.log(`Deployed hoprDistributor: ${hoprDistributor.address}`)
  addresses['HoprDistributor'] = hoprDistributor.address

  console.log('Updating addresses.json')
  await updateAddresses(network.name, addresses)
  console.log(`Updated addresses.json for ${network.name} network`)
}

export default main
