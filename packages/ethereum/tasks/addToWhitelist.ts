import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { utils } from 'ethers'

export type AddToWhitelistOpts = {
  addresses: string[]
}

/**
 * Used by our E2E tests to add addresses into the mocked whitelist.
 */
async function main(
  opts: AddToWhitelistOpts,
  { network, ethers, deployments, getNamedAccounts }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  if (network.name !== 'hardhat') {
    console.error('AddToWhitelist only works in a hardhat network.')
    process.exit(1)
  }

  let hoprDummyProxyAddress: string
  try {
    const contract = await deployments.get('HoprDummyProxyForNetworkRegistry')
    hoprDummyProxyAddress = contract.address
  } catch {
    console.error('HoprDummyProxyForNetworkRegistry contract has not been deployed. Deploy the contract and run again.')
    process.exit(1)
  }

  if (!opts.addresses.some((a) => !utils.isAddress(a))) {
    console.error(`Given address list '${opts.addresses.join(',')}' contains an invalid address.`)
    process.exit(1)
  }

  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const hoprDummyProxy = (await ethers.getContractFactory('HoprDummyProxyForNetworkRegistry'))
    .connect(deployer)
    .attach(hoprDummyProxyAddress)

  try {
    await (await hoprDummyProxy.ownerBatchAddAccounts(opts.addresses)).wait()
  } catch (error) {
    console.error('Failed to add account with error:', error)
    process.exit(1)
  }
}

export default main
