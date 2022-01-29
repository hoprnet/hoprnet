import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'

/**
 * Display unlocked accounts alongside with how much
 * ETH / HOPR they have.
 */
async function main(
  _params,
  { network, ethers, deployments }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  const contract = await deployments.get('HoprToken')

  const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(contract.address)

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
