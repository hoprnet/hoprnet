import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { utils } from 'ethers'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { convertPubKeyFromB58String } from '@hoprnet/hopr-utils'
import { HoprToken__factory } from '../types'
import { getContract } from './utils/contracts'

const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash) => {
    if (error) {
      console.log(`Error: ${error}`)
    }
    console.log(`transactionHash: ${transactionHash}`)
  })

const nativeAddress = async (hoprAddress) => {
  const nodePeerPubkey = await convertPubKeyFromB58String(hoprAddress)
  return utils.computeAddress(utils.arrayify(nodePeerPubkey.marshal()))
}

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(
  { address, amount, ishopraddress }: { address: string; amount: string; ishopraddress: boolean },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  if (network.name !== 'localhost') {
    console.error('🌵 Faucet is only valid in localhost network')
    return
  }

  let hoprTokenAddress: string
  try {
    const contract = await getContract(network.name, 'HoprToken')
    hoprTokenAddress = contract.address
  } catch {
    console.error('⛓  You need to ensure the network deployed the contracts')
    return
  }

  console.log('🚰 Starting local faucet task')
  const etherAmount = '1.0'
  const signer = ethers.provider.getSigner()
  const minterWallet = new ethers.Wallet(NODE_SEEDS[0], ethers.provider)
  const nodeAddress = ishopraddress ? await nativeAddress(address) : address
  const tx = {
    to: nodeAddress,
    value: ethers.utils.parseEther(etherAmount)
  }
  const hoprToken = HoprToken__factory.connect(hoprTokenAddress, ethers.provider).connect(minterWallet)

  console.log(`💧💰 Sending ${etherAmount} ETH to ${nodeAddress} on network ${network.name}`)
  await send(signer, tx)

  console.log(`💧🟡 Sending ${ethers.utils.formatEther(amount)} HOPR to ${nodeAddress} on network ${network.name}`)
  await hoprToken.mint(nodeAddress, amount, ethers.constants.HashZero, ethers.constants.HashZero, {
    from: minterWallet.getAddress(),
    gasLimit: 200e3
  })
}

export default main
