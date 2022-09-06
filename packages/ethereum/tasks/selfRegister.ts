import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { Signer, Wallet } from 'ethers'
import type { HoprNetworkRegistry } from '../src/types'

export type SelfRegisterOpts = {
  task: 'add' | 'remove'
  peerIds: string
  privatekey: string // private key of the caller
}

/**
 * Used by developers in testnet to register or deregister a node
 */
async function main(
  opts: SelfRegisterOpts,
  { ethers, deployments, environment }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  const hoprNetworkRegistryAddress = (await deployments.get('HoprNetworkRegistry')).address

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

  console.log('Signer Address', signerAddress)

  const hoprNetworkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry'))
    .connect(signer)
    .attach(hoprNetworkRegistryAddress) as HoprNetworkRegistry

  try {
    if (opts.task === 'add') {
      const peerIds = opts.peerIds.split(',')
      await (await hoprNetworkRegistry.selfRegister(peerIds)).wait()
    } else if (opts.task === 'remove') {
      const peerIds = opts.peerIds.split(',')
      await (await hoprNetworkRegistry.selfDeregister(peerIds)).wait()
    } else {
      throw Error(`Task "${opts}" not available.`)
    }
  } catch (error) {
    console.error('Failed to add account with error:', error)
    process.exit(1)
  }
}

export default main
