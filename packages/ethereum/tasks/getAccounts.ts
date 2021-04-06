import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { getAddresses } from '../chain'
import { HoprToken__factory } from '../types'

/**
 * Display unlocked accounts alongside with how much
 * ETH / HOPR they have.
 */
async function main(_params, { network, ethers }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const addresses = getAddresses()
  if (!addresses[network.name]) throw Error(`cannot find HoprToken address for network ${network.name}`)
  const hoprToken = HoprToken__factory.connect(addresses[network.name].HoprToken, ethers.provider)

  console.log('Running task "accounts" with config:', {
    network: network.name
  })

  const accounts = await ethers.getSigners()
  const nativeBalances = await Promise.all(
    accounts.map(async (account) => {
      const amount = await account.getBalance()
      return ethers.utils.formatEther(amount)
    })
  )
  const hoprBalances = await Promise.all(
    accounts.map(async (account) => {
      const amount = await hoprToken.balanceOf(account.address)
      return ethers.utils.formatEther(amount)
    })
  )

  console.table(
    accounts.map((account, i) => {
      return {
        account,
        native: nativeBalances[i],
        hopr: hoprBalances[i]
      }
    })
  )
}

export default main
