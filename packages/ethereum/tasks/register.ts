import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { type Signer, utils, Wallet } from 'ethers'
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
      privatekey?: string // private key of the caller
    }
  | {
      task: 'remove'
      nativeAddresses?: string
      peerIds: string
      privatekey?: string // private key of the caller
    }
  | {
      task: 'disable' | 'enable'
      privatekey?: string // private key of the caller
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

  let hoprProxyAddress: string
  let hoprNetworkRegistryAddress: string
  try {
    hoprProxyAddress = (await deployments.get('HoprNetworkRegistryProxy')).address
    hoprNetworkRegistryAddress = (await deployments.get('HoprNetworkRegistry')).address
  } catch {
    console.error(
      'HoprNetworkRegistry or HoprDummyProxyForNetworkRegistry contract has not been deployed. Deploy the contract and run again.'
    )
    process.exit(1)
  }

  let provider
  if (environment == 'hardhat-localhost') {
    // we use a custom ethers provider here instead of the ethers object from the
    // hre which is managed by hardhat-ethers, because that one seems to
    // run its own in-memory hardhat instance, which is undesirable
    provider = new ethers.providers.JsonRpcProvider()
  } else {
    provider = ethers.provider
  }

  let signer: Signer
  if (!opts.privatekey) {
    signer = provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, provider)
  }
  const signerAddress = await signer.getAddress()
  console.log('Signer Address (register task)', signerAddress)

  const hoprProxy =
    network.tags.development
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

  console.log(`HoprProxy Address (register task) ${hoprProxyAddress}`)
  console.log(`HoprNetworkRegistry Address (register task) ${hoprNetworkRegistryAddress} is ${isEnabled}`)

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

      // in staging or production, register by owner; in non-staging environment, add addresses directly to proxy
      if (network.tags.development) {
        await (await (hoprProxy as HoprDummyProxyForNetworkRegistry).ownerBatchAddAccounts(nativeAddresses)).wait()
      }
      await (await hoprNetworkRegistry.ownerRegister(nativeAddresses, peerIds)).wait()
    } else if (opts.task === 'remove') {
      const peerIds = opts.peerIds.split(',')

      // in staging or production (where "HoprStakingProxyForNetworkRegistry" is used), deregister; in non-staging environment (where "HoprDummyProxyForNetworkRegistry" is used), remove addresses directly from proxy
      if (network.tags.development) {
        let nativeAddresses

        if (opts.nativeAddresses) {
          nativeAddresses = opts.nativeAddresses.split(',')
          // ensure all native addresses are valid
          if (nativeAddresses.some((a) => !utils.isAddress(a))) {
            console.error(`Given address list '${nativeAddresses.join(',')}' contains an invalid address.`)
            process.exit(1)
          }

          // ensure lists match in length
          if (nativeAddresses.length !== peerIds.length) {
            console.error('Given native and multiaddress lists do not match in length.')
            process.exit(1)
          }
          // remove account from dummy proxy
          await (await (hoprProxy as HoprDummyProxyForNetworkRegistry).ownerBatchRemoveAccounts(nativeAddresses)).wait()
        } else {
          console.error(`Must provide addresses in ownerDeregister in ${environment} (where dummy proxy is used)`)
          process.exit(1)
        }
      }
      await (await hoprNetworkRegistry.ownerDeregister(peerIds)).wait()
    } else if (opts.task === 'enable' && !isEnabled) {
      await (await hoprNetworkRegistry.enableRegistry()).wait()
    } else if (opts.task === 'disable' && isEnabled) {
      await (await hoprNetworkRegistry.disableRegistry()).wait()
    } else {
      throw Error(`Task "${opts.task}" not available.`)
    }
  } catch (error) {
    console.error('Failed to interact with network registry with error:', error)
    process.exit(1)
  }
}

export default main
