import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { HoprToken } from '../types'

import { getContractData, Networks } from '..'

/**
 * Display unlocked accounts alongside with how much
 * ETH / HOPR they have.
 */
async function main(_params, { network, ethers }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const contracts = getContractData(network.name as Networks, 'HoprToken')
  if (!contracts?.[network.name]) throw Error(`cannot find HoprToken address for network ${network.name}`)
  const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(
    contracts[network.name].HoprToken.address
  ) as HoprToken

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
