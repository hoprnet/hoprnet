import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { HoprToken__factory } from '../types'

/**
 * Funds all unlocked accounts with HOPR
 */
async function main(
  { address, amount, accountsToFund }: { address: string; amount: string; accountsToFund: number },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  console.log({
    address,
    network: network.name
  })
  const signers = await ethers.getSigners()
  const signer = signers[0]
  const accounts = signers.map((signer) => signer.address).slice(0, accountsToFund)
  const hoprToken = HoprToken__factory.connect(address, ethers.provider).connect(signer)

  console.log('Running task "fund" with config:', {
    network: network.name,
    address,
    amount,
    accounts
  })

  for (const account of accounts) {
    await hoprToken.mint(account, amount, ethers.constants.HashZero, ethers.constants.HashZero, {
      from: signer.address,
      gasLimit: 200e3
    })

    console.log(`Funded: ${account}`)
  }
}

export default main
