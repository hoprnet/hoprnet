import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { HoprToken__factory } from '../types'

const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash) => {
    if (error) {
      console.log(`Error: ${error}`)
    }
    console.log(`transactionHash: ${transactionHash}`)
  })

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(
  { address, amount }: { address: string; amount: string },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  console.log('🚰 Starting faucet task', {
    address,
    network: network.name
  })
  const etherAmount = '1'
  const signer = ethers.provider.getSigner()
  const tx = {
    to: address,
    value: ethers.utils.parseEther(etherAmount)
  }
  const minterWallet = new ethers.Wallet(NODE_SEEDS[0], ethers.provider)
  const hoprToken = HoprToken__factory.connect(address, ethers.provider).connect(minterWallet)

  console.log(`💧💰 Sending ${etherAmount} ETH to ${address} on network ${network.name}`)
  await send(signer, tx)

  console.log(`💧🟡 Sending ${amount} HOPR to ${address} on network ${network.name}`)
  await hoprToken.mint(address, amount, ethers.constants.HashZero, ethers.constants.HashZero, {
    from: minterWallet.getAddress(),
    gasLimit: 200e3
  })

  await hoprToken.balanceOf(address).then(console.log)
}

export default main
