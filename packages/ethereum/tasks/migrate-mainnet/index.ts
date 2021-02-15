import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { HoprTokenContract, HoprDistributorContract } from '../../types'
import transform, { getHoprDistributorParams, getSchedule, getAllocations } from './transform'
import { addresses as allAddresses, ContractNames } from '../../chain'
import updateAddresses from '../../utils/updateAddresses'

async function main(
  options: { task: string; schedule?: string; allocation?: string },
  { web3, network, artifacts }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const [deployer] = await web3.eth.getAccounts()
  const addresses: { [key in ContractNames]: string } = allAddresses[network.name]
  const HoprToken: HoprTokenContract = artifacts.require('HoprToken')
  const HoprDistributor: HoprDistributorContract = artifacts.require('HoprDistributor')

  console.log('Running task "migrate-mainnet" with config:', {
    deployer,
    network: network.name,
    task: options.task,
    schedule: options.schedule
  })

  if (options.task === 'deployToken') {
    const hoprToken = await HoprToken.new()

    await updateAddresses(network.name, { HoprToken: hoprToken.address })
  } else if (options.task === 'deployDist') {
    const hoprToken = await HoprToken.at(addresses.HoprToken)
    const params = await getHoprDistributorParams(network.name)
    const hoprDistributor = await HoprDistributor.new(hoprToken.address, params.startTime, params.maxMintAmount)

    await updateAddresses(network.name, { HoprDistributor: hoprDistributor.address })
  } else if (options.task === 'transform') {
    await transform()
  } else if (options.task === 'addSchedule') {
    const hoprDistributor = await HoprDistributor.at(addresses.HoprDistributor)
    const schedule = await getSchedule(options.schedule)

    await hoprDistributor.addSchedule(schedule.durations, schedule.percents, schedule.name)
  } else if (options.task === 'addAllocations') {
    const hoprDistributor = await HoprDistributor.at(addresses.HoprDistributor)
    const allocation = await getAllocations(options.allocation)

    await hoprDistributor.addAllocations(allocation.accounts, allocation.amounts, allocation.name)
  } else if (options.task === 'grantMinter') {
    const hoprToken = await HoprToken.at(addresses.HoprToken)
    const hoprDistributor = await HoprDistributor.at(addresses.HoprDistributor)
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()

    await hoprToken.grantRole(MINTER_ROLE, hoprDistributor.address)
  } else if (options.task === 'grantTokenAdmin') {
    const hoprToken = await HoprToken.at(addresses.HoprToken)
    const ADMIN_ROLE = await hoprToken.DEFAULT_ADMIN_ROLE()
    const hoprDistributor = await HoprDistributor.at(addresses.HoprDistributor)
    const params = await getHoprDistributorParams(network.name)

    await hoprToken.grantRole(ADMIN_ROLE, params.multisig)
    await hoprDistributor.transferOwnership(params.multisig)
  } else if (options.task === 'renounceTokenAdmin') {
    const hoprToken = await HoprToken.at(addresses.HoprToken)
    const ADMIN_ROLE = await hoprToken.DEFAULT_ADMIN_ROLE()
    const params = await getHoprDistributorParams(network.name)

    if (!hoprToken.hasRole(ADMIN_ROLE, params.multisig)) {
      throw Error('Multisig must be admin before renouncing role')
    }
    await hoprToken.renounceRole(ADMIN_ROLE, deployer)
  } else if (options.task === 'transferDistOwner') {
    const hoprDistributor = await HoprDistributor.at(addresses.HoprDistributor)
    const params = await getHoprDistributorParams(network.name)

    await hoprDistributor.transferOwnership(params.multisig)
  }
}

export default main
