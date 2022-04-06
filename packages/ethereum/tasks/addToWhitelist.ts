import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { utils } from 'ethers'

export type AddToWhitelistOpts = {
  nativeAddresses: string
  multiaddresses: string
}

/**
 * Used by our E2E tests to add addresses into the mocked whitelist.
 */
async function main(
  opts: AddToWhitelistOpts,
  { network, ethers, deployments, environment }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  if (network.name !== 'hardhat') {
    console.error('AddToWhitelist only works in a hardhat network.')
    process.exit(1)
  }

  let hoprDummyProxyAddress: string
  let hoprNetworkRegistryAddress: string
  try {
    hoprDummyProxyAddress = (await deployments.get('HoprDummyProxyForNetworkRegistry')).address
    hoprNetworkRegistryAddress = (await deployments.get('HoprNetworkRegistry')).address
  } catch {
    console.error(
      'HoprNetworkRegistry or HoprDummyProxyForNetworkRegistry contract has not been deployed. Deploy the contract and run again.'
    )
    process.exit(1)
  }

  const nativeAddresses = opts.nativeAddresses.split(',')
  const multiaddresses = opts.multiaddresses.split(',')

  // ensure lists match in length
  if (nativeAddresses.length !== multiaddresses.length) {
    console.error('Given native and multiaddress lists do not match in length.')
    process.exit(1)
  }

  // ensure all native addresses are valid
  if (nativeAddresses.some((a) => !utils.isAddress(a))) {
    console.error(`Given address list '${nativeAddresses.join(',')}' contains an invalid address.`)
    process.exit(1)
  }

  // we use a custom ethers provider here instead of the ethers object from the
  // hre which is managed by hardhat-ethers, because that one seems to
  // run its own in-memory hardhat instance, which is undesirable
  const provider = new ethers.providers.JsonRpcProvider()
  const signer = provider.getSigner()

  const hoprDummyProxy = (await ethers.getContractFactory('HoprDummyProxyForNetworkRegistry'))
    .connect(signer)
    .attach(hoprDummyProxyAddress)

  const hoprNetworkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry'))
    .connect(signer)
    .attach(hoprNetworkRegistryAddress)

  try {
    await (await hoprDummyProxy.ownerBatchAddAccounts(nativeAddresses)).wait()
    await (await hoprNetworkRegistry.ownerRegister(nativeAddresses, multiaddresses)).wait()
  } catch (error) {
    console.error('Failed to add account with error:', error)
    process.exit(1)
  }
}

export default main
