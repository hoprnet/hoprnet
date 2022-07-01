import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { utils } from 'ethers'
import type {
  HoprNetworkRegistry,
  HoprDummyProxyForNetworkRegistry,
  HoprStakingProxyForNetworkRegistry
} from '../src/types'

export type RegisterOpts =
  | {
      task: 'add'
      nativeAddresses: string
      peerIds: string
    }
  | {
      task: 'remove'
      nativeAddresses: string
    }
  | {
      task: 'disable' | 'enable'
    }

/**
 * Used by our E2E tests to interact with 'HoprNetworkRegistry' and 'HoprDummyProxyForNetworkRegistry'.
 * Can also be used by the CI to automate registery for cloud cluster nodes
 */
async function main(
  opts: RegisterOpts,
  { network, ethers, deployments, environment }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  if (network.name !== 'hardhat' || !network.tags.staging) {
    console.error('Register only works in a hardhat or staging network.')
    process.exit(1)
  }

  let hoprProxyAddress: string
  let hoprNetworkRegistryAddress: string
  try {
    hoprProxyAddress = !network.tags.staging
      ? (await deployments.get('HoprNetworkRegistryProxy')).address
      : (await deployments.get('HoprNetworkRegistryProxy')).address
    hoprNetworkRegistryAddress = (await deployments.get('HoprNetworkRegistry')).address
  } catch {
    console.error(
      'HoprNetworkRegistry or HoprDummyProxyForNetworkRegistry contract has not been deployed. Deploy the contract and run again.'
    )
    process.exit(1)
  }

  // we use a custom ethers provider here instead of the ethers object from the
  // hre which is managed by hardhat-ethers, because that one seems to
  // run its own in-memory hardhat instance, which is undesirable
  const provider = new ethers.providers.JsonRpcProvider()
  const signer = provider.getSigner()

  const hoprProxy = !network.tags.staging
    ? ((await ethers.getContractFactory('HoprDummyProxyForNetworkRegistry'))
        .connect(signer)
        .attach(hoprProxyAddress) as HoprDummyProxyForNetworkRegistry)
    : ((await ethers.getContractFactory('HoprStakingProxyForNetworkRegistry'))
        .connect(signer)
        .attach(hoprProxyAddress) as HoprStakingProxyForNetworkRegistry)

  const hoprNetworkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry'))
    .connect(signer)
    .attach(hoprNetworkRegistryAddress) as HoprNetworkRegistry
  const isEnabled = await hoprNetworkRegistry.enabled()

  try {
    if (opts.task === 'add') {
      const nativeAddresses = opts.nativeAddresses.split(',')
      const peerIds = opts.peerIds.split(',')

      // ensure lists match in length
      if (nativeAddresses.length !== peerIds.length) {
        console.error('Given native and multiaddress lists do not match in length.')
        process.exit(1)
      }

      // ensure all native addresses are valid
      if (nativeAddresses.some((a) => !utils.isAddress(a))) {
        console.error(`Given address list '${nativeAddresses.join(',')}' contains an invalid address.`)
        process.exit(1)
      }

      // in staging account, register by owner; in non-stagin environment, add addresses directly to proxy
      if (!network.tags.staging) {
        await (await (hoprProxy as HoprDummyProxyForNetworkRegistry).ownerBatchAddAccounts(nativeAddresses)).wait()
      }
      await (await hoprNetworkRegistry.ownerRegister(nativeAddresses, peerIds)).wait()
    } else if (opts.task === 'remove') {
      const nativeAddresses = opts.nativeAddresses.split(',')
      // ensure all native addresses are valid
      if (nativeAddresses.some((a) => !utils.isAddress(a))) {
        console.error(`Given address list '${nativeAddresses.join(',')}' contains an invalid address.`)
        process.exit(1)
      }

      // in staging account, deregister; in non-stagin environment, remove addresses directly from proxy
      if (!network.tags.staging) {
        await (await (hoprProxy as HoprDummyProxyForNetworkRegistry).ownerBatchRemoveAccounts(nativeAddresses)).wait()
      }
      await (await hoprNetworkRegistry.ownerDeregister(nativeAddresses)).wait()
    } else if (opts.task === 'enable' && !isEnabled) {
      await (await hoprNetworkRegistry.enableRegistry()).wait()
    } else if (opts.task === 'disable' && isEnabled) {
      await (await hoprNetworkRegistry.disableRegistry()).wait()
    } else {
      throw Error(`Task "${opts.task}" not available.`)
    }
  } catch (error) {
    console.error('Failed to add account with error:', error)
    process.exit(1)
  }
}

export default main
