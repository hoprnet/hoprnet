import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { utils } from 'ethers'
import { convertPubKeyFromB58String } from '@hoprnet/hopr-utils'
import { HoprToken__factory } from '../types'
import { getContractData, Networks } from '..'

const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash) => {
    if (error) {
      console.log(`Error: ${error}`)
    }
    console.log(`transactionHash: ${transactionHash}`)
  })

const nativeAddress = async (hoprAddress) => {
  const nodePeerPubkey = await convertPubKeyFromB58String(hoprAddress)
  return utils.computeAddress(nodePeerPubkey.marshal())
}

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(
  { address, amount, ishopraddress }: { address: string; amount: string; ishopraddress: boolean },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  if (network.tags.development) {
    console.error('🌵 Faucet is only valid in a development network')
    return
  }

  let hoprTokenAddress: string
  try {
    const contract = getContractData(network.name as Networks, 'HoprToken')
    hoprTokenAddress = contract.address
  } catch {
    console.error('⛓  You need to ensure the network deployed the contracts')
    return
  }

  const etherAmount = '1.0'
  const signer = ethers.provider.getSigner()
  const nodeAddress = ishopraddress ? await nativeAddress(address) : address
  const tx = {
    to: nodeAddress,
    value: ethers.utils.parseEther(etherAmount)
  }
  const hoprToken = HoprToken__factory.connect(hoprTokenAddress, ethers.provider).connect(signer)

  console.log(`💧💰 Sending ${etherAmount} ETH to ${nodeAddress} on network ${network.name}`)
  await send(signer, tx)

  console.log(`💧🟡 Sending ${ethers.utils.formatEther(amount)} HOPR to ${nodeAddress} on network ${network.name}`)
  await hoprToken.mint(nodeAddress, amount, ethers.constants.HashZero, ethers.constants.HashZero, {
    gasLimit: 200e3
  })
}

export default main
