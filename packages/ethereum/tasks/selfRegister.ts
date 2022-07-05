import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { Signer, Wallet } from 'ethers'
import type { HoprNetworkRegistry } from '../src/types'

export type SelfRegisterOpts =
  | {
      task: 'add'
      peerId: string
      privatekey: string // private key of the caller
    }
  | {
      task: 'remove'
      privatekey: string // private key of the caller
    }

/**
 * Used by developers in testnet to register or deregister a node
 */
async function main(
  opts: SelfRegisterOpts,
  { ethers, deployments }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  const hoprNetworkRegistryAddress = (await deployments.get('HoprNetworkRegistry')).address

  let signer: Signer
  if (!opts.privatekey) {
    signer = ethers.provider.getSigner()
  } else {
    signer = new Wallet(opts.privatekey, ethers.provider)
  }
  const signerAddress = await signer.getAddress()

  console.log('Signer Address', signerAddress)

  const hoprNetworkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry'))
    .connect(signer)
    .attach(hoprNetworkRegistryAddress) as HoprNetworkRegistry

  try {
    if (opts.task === 'add') {
      const peerId = opts.peerId

      await (await hoprNetworkRegistry.selfRegister(peerId)).wait()
    } else if (opts.task === 'remove') {
      await (await hoprNetworkRegistry.selfDeregister()).wait()
    } else {
      throw Error(`Task "${opts}" not available.`)
    }
  } catch (error) {
    console.error('Failed to add account with error:', error)
    process.exit(1)
  }
}

export default main
