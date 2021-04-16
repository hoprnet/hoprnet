import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import { getContracts } from '../chain'
import { HoprToken__factory } from '../types'
import { Logger } from '@hoprnet/hopr-utils'

const log: Logger = Logger.getLogger('hoprd.tasks.getAccounts')

/**
 * Display unlocked accounts alongside with how much
 * ETH / HOPR they have.
 */
async function main(_params, { network, ethers }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  const contracts = getContracts()
  if (!contracts?.[network.name]) throw Error(`cannot find HoprToken address for network ${network.name}`)
  const hoprToken = HoprToken__factory.connect(contracts[network.name].HoprToken.address, ethers.provider)

  log.info('Running task "accounts" with config:', {
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

  log.info(
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
